use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use dashmap::{DashMap, DashSet};
use tokio::sync::mpsc::{Receiver, channel, Sender};
use crate::{CommandResponse, KvError};
use tracing::{instrument, warn, debug};
use crate::value::Value;

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

    fn unsubscribe(self, name: String, id: u32) -> Result<u32, KvError> {
        todo!()
    }

    fn publish(self, name: String, value: Arc<CommandResponse>) {
        todo!()
    }
}

impl Broadcaster {
    
}





