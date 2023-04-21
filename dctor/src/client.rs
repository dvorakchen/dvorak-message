use super::dctor::{Dctor, Inbox};
use async_trait::async_trait;
use std::{io::Write, net::TcpStream};
use tokio::sync::mpsc::{self, Sender};

pub(crate) enum ClientMessage {
    /// representing there is a message need send,
    /// tuple parameters: (sender, message)
    ReceiveMessage(String, String),
    /// terminate current client
    Terminate,
}

pub(crate) struct Client {
    tcp_stream: TcpStream,
    inbox: Inbox<<Self as Dctor>::InboxItem>,
}

impl Client {
    pub fn new(tcp_stream: TcpStream) -> (Self, Sender<<Self as Dctor>::InboxItem>) {
        let (tx, rx) = mpsc::channel(100);
        (
            Client {
                tcp_stream,
                inbox: rx,
            },
            tx,
        )
    }
}

#[async_trait]
impl Dctor for Client {
    type InboxItem = ClientMessage;

    async fn listen(&mut self) {
        use ClientMessage::*;

        while let Some(msg) = self.inbox.recv().await {
            match msg {
                ReceiveMessage(sender, message) => {
                    let data = format!("{{ sender: '{sender}', message: '{message}' }}");
                    let buf = data.as_bytes();
                    self.tcp_stream.write_all(buf).unwrap();
                }
                Terminate => {
                    return;
                }
            }
        }
    }
}
