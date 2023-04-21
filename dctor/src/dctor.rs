use std::sync::mpsc::{Sender, Receiver};

#[async_trait]
pub(crate) trait Dctor {
    type InboxItem;

    async fn listen(&mut self);
}

pub(crate)type Inbox<T> = Receiver<T>;