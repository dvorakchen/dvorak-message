use super::{
    dctor::{Dctor, Inbox},
    supervisor::SupervisorSender,
};
use async_trait::async_trait;
use dvorak_message::message::Message;
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::mpsc::{self, Sender},
};

#[derive(Debug)]
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
    supervisor_sender: SupervisorSender,
}

impl Client {
    pub fn new(
        tcp_stream: TcpStream,
        supervisor_sender: SupervisorSender,
    ) -> (Self, Sender<<Self as Dctor>::InboxItem>) {
        let (tx, rx) = mpsc::channel(100);
        (
            Client {
                tcp_stream,
                inbox: rx,
                supervisor_sender,
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

        tokio::select! {
            msg = Message::read_from(&mut self.tcp_stream) => {
                // let msg = msg.map_err(|_| "read from client failed")
                //     .ok_or_else(|| "read from client failed");
            },
            msg = self.inbox.recv() => {
                if let Some(msg) = msg {
                    match msg {
                        ReceiveMessage(sender, message) => {
                            let data = format!("{{ sender: '{sender}', message: '{message}' }}");
                            let buf = data.as_bytes();
                            self.tcp_stream.write_all(buf).await.unwrap();
                        }
                        Terminate => {
                            return;
                        }
                    }
                }
            }
        }
    }
}
