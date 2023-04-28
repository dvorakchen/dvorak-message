use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

#[async_trait]
pub(crate) trait Dctor {
    type InboxItem;

    async fn listen(&mut self);
}

pub(crate) type Inbox<T> = Receiver<T>;
