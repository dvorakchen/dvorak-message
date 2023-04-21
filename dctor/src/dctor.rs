use std::sync::mpsc::{Sender, Receiver};

pub(crate) trait Dctor {
    type InboxItem;

    fn listen(&mut self);
}

pub(crate)type Inbox<T> = Receiver<T>;