use std::error::Error;
use tokio::sync::mpsc::Sender;
use std::fmt::Debug;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ResultPublisher<R: Clone + Sync + Send, E: Error + Sync + Send>: Sync + Send {
    async fn notify(&self, result: R) -> Result<R, E>;
}

pub struct TokioResultPublisher<R> {
    sender: Sender<R>,
}

impl<R> TokioResultPublisher<R> {
    pub fn new(sender: Sender<R>) -> TokioResultPublisher<R> {
        TokioResultPublisher {
            sender
        }
    }
}

#[async_trait]
impl<R: Clone + Sync + Send + Debug, E: Error + Sync + Send> ResultPublisher<R, E> for TokioResultPublisher<R> {
    async fn notify(&self, result: R) -> Result<R, E> {
        let _ = self.sender.send(result.clone()).await;
        Ok(result)
    }
}
