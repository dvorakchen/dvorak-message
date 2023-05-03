use super::{
    dctor::{Dctor, Inbox},
    supervisor::{SupervisorMessage, SupervisorSender},
};
use async_trait::async_trait;
use dvorak_message::message::{Message, MessageType};
use tokio::{
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
        println!("Client construct");
        (
            Client {
                tcp_stream,
                inbox: rx,
                supervisor_sender,
            },
            tx,
        )
    }

    /// handle incoming message
    ///
    /// # Return
    /// is terminate the listen?
    async fn handle_incoming_message(&mut self, message: Message) -> bool {
        match &message.message_type {
            MessageType::Text(data) => {
                println!("Received type: Text");
                let receiver = message.receiver.clone();
                let sender = message.username.clone();

                self.supervisor_sender
                    .send(SupervisorMessage::Message {
                        sender,
                        receiver: receiver.clone(),
                        message: data.clone(),
                    })
                    .await
                    .unwrap();

                return false;
            }
            MessageType::Logout => {
                println!("Received type: Logout");

                let username = message.username.clone();

                self.supervisor_sender
                    .send(SupervisorMessage::DisconnectClient(username))
                    .await
                    .unwrap();
                return true;
            }
            _ => {
                println!("Received type: other");
                return false;
            }
        }
    }
}

#[async_trait]
impl Dctor for Client {
    type InboxItem = ClientMessage;

    async fn listen(&mut self) {
        use ClientMessage::*;

        println!("Client listening...");

        loop {
            tokio::select! {
                msg = Message::read_from(&mut self.tcp_stream) => {
                    if msg.is_err() {
                        continue;
                    }
                    let msg = msg.unwrap();
                    if msg.is_none() {
                        continue;
                    }
                    let message = msg.unwrap();
                    println!("Client received message");
                    let is_break = self.handle_incoming_message(message).await;

                    if is_break {
                        break;
                    }
                },
                msg = self.inbox.recv() => {
                    if let Some(msg) = msg {
                        match msg {
                            ReceiveMessage(sender, message) => {
                                let message = Message::new(MessageType::Text(message), sender, String::from("Self"));
                                Message::send(&mut self.tcp_stream, message).await.unwrap();
                                // let data = format!("{{ sender: '{sender}', message: '{message}' }}");
                                // println!("Debug: Client in Server ReceiveMessage: {data}");
                                // let buf = data.as_bytes();
                                // self.tcp_stream.write_all(buf).await.unwrap();
                            }
                            Terminate => {
                                println!("Client terminated.");
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}
