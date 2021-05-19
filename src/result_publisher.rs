use std::error::Error;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait ResultPublisher<R: Clone + Sync + Send, E: Error>: Sync + Send {
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
impl<R: Clone + Sync + Send, E: Error> ResultPublisher<R, E> for TokioResultPublisher<R> {
    async fn notify(&self, result: R) -> Result<R, E> {
        let _ = self.sender.send(result.clone()).await;
        Ok(result)
    }
}

pub fn create_tokio_publisher<T>(tx: Sender<T>) -> TokioResultPublisher<T> {
    TokioResultPublisher::new(tx)
}