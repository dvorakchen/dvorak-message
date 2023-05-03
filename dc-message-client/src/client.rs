use std::sync::Arc;

use dvorak_message::message::{Message, MessageType};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::input::Input;

type Username = String;

pub(crate) struct Client {
    tcp_stream: TcpStream,
    inbox: Receiver<ClientMessage>,
    username: Username,
    receiver: Option<Username>,
    input_handler: Input,
}

#[derive(Debug)]
pub(crate) enum ClientMessage {
    Text(String),
    To(String),
    Quit,
}

pub(crate) type ClientSender = Arc<Sender<ClientMessage>>;

impl Client {
    pub fn new(username: Username, tcp_stream: TcpStream) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let sender = Arc::new(tx);

        Client {
            tcp_stream,
            inbox: rx,
            username,
            receiver: None,
            input_handler: Input::new(sender),
        }
    }

    pub async fn listen(&mut self) {

        self.input_handler.listen().await;

        'listen: loop {
            tokio::select! {
                client_message = self.inbox.recv() => {
                    if let Some(msg) = client_message {
                        match msg {
                            ClientMessage::Text(data) => {
                                if let Some(receiver_name) = &self.receiver {
                                    let message = Message::new(
                                        MessageType::Text(data),
                                        self.username.clone(),
                                        receiver_name.clone(),
                                    );
                                    Message::send(&mut self.tcp_stream, message).await.unwrap();
                                }
                            },
                            ClientMessage::Quit => {
                                println!("Received instruct: Quit");
                                break 'listen;
                            },
                            ClientMessage::To(username) => {
                                println!("Change receiver: {username}");
                                self.receiver = Some(username)
                            }
                        }
                    }
                },
                message = async {
                    Message::read_from(&mut self.tcp_stream).await.unwrap()

                } => {
                    if message.is_none() {
                        continue;
                    }

                    let bytes = message.unwrap().message_type.as_bytes();
                    let data: String = String::from_utf8(bytes.to_vec()).unwrap();
                    println!("{data}");
                }
            }
        }
    }
}
