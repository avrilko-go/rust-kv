use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use dashmap::{DashMap, DashSet};
use tokio::sync::mpsc::{Receiver, channel, Sender};
use crate::{CommandResponse, KvError, Value};
use tracing::{instrument, warn, debug, info};

const BROADCAST_CAPACITY: usize = 128;

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

fn get_next_subscription_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

pub trait Topic: Send + Sync + 'static {
    fn subscribe(self, name: String) -> Receiver<Arc<CommandResponse>>;

    fn unsubscribe(self, name: String, id: u32) -> Result<u32, KvError>;

    fn publish(self, name: String, value: Arc<CommandResponse>);
}

#[derive(Default)]
pub struct Broadcaster {
    topics: DashMap<String, DashSet<u32>>,
    subscriptions: DashMap<u32, Sender<Arc<CommandResponse>>>,
}

impl Topic for Arc<Broadcaster> {
    #[instrument(name = "topic_subscribe", skip_all)]
    fn subscribe(self, name: String) -> Receiver<Arc<CommandResponse>> {
        let id = {
            let entry = self.topics.entry(name).or_default();
            let id = get_next_subscription_id();
            entry.value().insert(id);
            id
        };

        let (tx, rx) = channel(BROADCAST_CAPACITY);
        let v: Value = (id as i64).into();
        let tx1 = tx.clone();

        tokio::spawn(async move {
            if let Err(e) = tx1.send(Arc::new(v.into())).await {
                warn!("Failed to send subscription id: {}. Error: {:?}", id, e);
            }
        });
        self.subscriptions.insert(id, tx);
        debug!("Subscription {} is added", id);
        rx
    }

    #[instrument(name = "topic_unsubscribe", skip_all)]
    fn unsubscribe(self, name: String, id: u32) -> Result<u32, KvError> {
        match self.remove_subscription(name, id) {
            Some(id) => Ok(id),
            None => Err(KvError::NotFound(format!("subscription {}", id))),
        }
    }

    #[instrument(name = "topic_publish", skip_all)]
    fn publish(self, name: String, value: Arc<CommandResponse>) {
        tokio::spawn(async move {
            let mut ids = vec![];
            if let Some(topic) = self.topics.get(&name) {
                let subscriptions = topic.value().clone();
                drop(topic);
                for id in subscriptions.into_iter() {
                    if let Some(tx) = self.subscriptions.get(&id) {
                        if let Err(e) = tx.send(value.clone()).await {
                            warn!("Publish to {} failed! error: {:?}", id, e);
                            ids.push(id);
                        }
                    }
                }
            }

            for id in ids {
                self.remove_subscription(name.clone(), id);
            }
        });
    }
}

impl Broadcaster {
    fn remove_subscription(&self, name: String, id: u32) -> Option<u32> {
        if let Some(v) = self.topics.get_mut(&name) {
            v.remove(&id);
            if v.is_empty() {
                info!("Topic: {:?} is deleted", &name);
                drop(v);
                self.topics.remove(&name);
            }
        }

        debug!("Subscription {} is removed!", id);
        self.subscriptions.remove(&id).map(|(id, _)| id)
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_res_ok;
    use super::*;

    #[tokio::test]
    async fn pub_sub_should_work() {
        let b = Arc::new(Broadcaster::default());
        let lobby = "lobby".to_string();

        let mut stream1 = b.clone().subscribe(lobby.clone());
        let mut stream2 = b.clone().subscribe(lobby.clone());

        let v: Value = "hello".into();
        b.clone().publish(lobby.clone(), Arc::new(v.clone().into()));

        let id1 = get_id(&mut stream1).await;
        let id2 = get_id(&mut stream2).await;
        assert_ne!(id1, id2);

        let res1 = stream1.recv().await.unwrap();
        let res2 = stream2.recv().await.unwrap();
        assert_eq!(res1, res2);

        assert_res_ok(&res1, &[v.clone()], &[]);

        let result = b.clone().unsubscribe(lobby.clone(), id1 as _).unwrap();
        assert_eq!(result, id1 as _);

        let v: Value = "world".into();
        b.clone().publish(lobby.clone(), Arc::new(v.clone().into()));

        assert!(stream1.recv().await.is_none());

        let res2 = stream2.recv().await.unwrap();
        assert_res_ok(&res2, &[v.clone()], &[]);
    }

    pub async fn get_id(res: &mut Receiver<Arc<CommandResponse>>) -> u32 {
        let id: i64 = res.recv().await.unwrap().as_ref().try_into().unwrap();
        id as _
    }
}





