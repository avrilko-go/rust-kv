use futures::{Stream, stream};
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;
use crate::{CommandResponse, Publish, Subscribe, Unsubscribe};
use crate::topic::Topic;

pub type StreamingResponse = Pin<Box<dyn Stream<Item=Arc<CommandResponse>> + Send>>;

pub trait TopicService {
    fn execute(self, topic: impl Topic) -> StreamingResponse;
}

impl TopicService for Subscribe {
    fn execute(self, topic: impl Topic) -> StreamingResponse {
        let rx = topic.subscribe(self.topic);
        Box::pin(ReceiverStream::new(rx))
    }
}

impl TopicService for Unsubscribe {
    fn execute(self, topic: impl Topic) -> StreamingResponse {
        let res = match topic.unsubscribe(self.topic, self.id) {
            Ok(_) => CommandResponse::ok(),
            Err(e) => e.into()
        };

        Box::pin(stream::once(async { Arc::new(res) }))
    }
}

impl TopicService for Publish {
    fn execute(self, topic: impl Topic) -> StreamingResponse {
        topic.publish(self.topic, Arc::new(self.data.into()));
        Box::pin(stream::once(async { Arc::new(CommandResponse::ok()) }))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio_stream::StreamExt;
    use crate::{assert_res_error, assert_res_ok, CommandRequest, dispatch_stream};
    use crate::topic::Broadcaster;
    use super::*;


    #[tokio::test]
    async fn dispatch_publish_should_work() {
        let topic = Arc::new(Broadcaster::default());
        let cmd = CommandRequest::new_publish("lobby", vec!["hello".into()]);
        let mut res = dispatch_stream(cmd, topic);
        let data = res.next().await.unwrap();
        assert_res_ok(&data, &[], &[]);
    }

    #[tokio::test]
    async fn dispatch_subscribe_should_work() {
        let topic = Arc::new(Broadcaster::default());
        let cmd = CommandRequest::new_subscribe("lobby");
        let mut res = dispatch_stream(cmd, topic);
        let id = get_id(&mut res).await;
        assert!(id > 0);
    }

    #[tokio::test]
    async fn dispatch_subscribe_abnormal_quit_should_be_removed_on_next_publish() {
        let topic = Arc::new(Broadcaster::default());
        let id = {
            let cmd = CommandRequest::new_subscribe("lobby");
            let mut res = dispatch_stream(cmd, topic.clone());
            let id = get_id(&mut res).await;
            drop(res);
            id
        };

        let cmd = CommandRequest::new_publish("lobby", vec!["hello".into()]);
        dispatch_stream(cmd, topic.clone());
        tokio::time::sleep(Duration::from_millis(10)).await;

        let result = topic.unsubscribe("lobby".into(), id);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn dispatch_unsubscribe_should_work() {
        let topic = Arc::new(Broadcaster::default());
        let cmd = CommandRequest::new_subscribe("lobby");

        let mut res = dispatch_stream(cmd, topic.clone());
        let id = get_id(&mut res).await;

        let cmd = CommandRequest::new_unsubscribe("lobby", id as _);
        let mut res = dispatch_stream(cmd, topic);
        let data = res.next().await.unwrap();
        assert_res_ok(&data, &[], &[]);
    }

    #[tokio::test]
    async fn dispatch_unsubscribe_random_id_should_error() {
        let topic = Arc::new(Broadcaster::default());

        let cmd = CommandRequest::new_unsubscribe("lobby", 9527);
        let mut res = dispatch_stream(cmd, topic);
        let data = res.next().await.unwrap();

        assert_res_error(&data, 404, "Not found: subscription 9527");
    }

    pub async fn get_id(res: &mut StreamingResponse) -> u32 {
        let id: i64 = res.next().await.unwrap().as_ref().try_into().unwrap();
        id as u32
    }
}