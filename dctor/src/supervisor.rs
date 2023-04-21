use crate::client::Client;

use super::client::ClientMessage;
use super::dctor::Dctor;
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

/// Actor Message for ClientSupervisor
pub enum SupervisorMessage {
    /// representing a new client established
    /// tuple parameters: (client username, TcpStream)
    NewClient(String, TcpStream),
    /// client send message to another client
    Message {
        /// username who send this message
        sender: String,
        /// username who receive this message
        receiver: String,
        /// message for sending
        message: String,
    },
    /// representing client disconnecting to server
    DisconnectClient(String),
}

struct StoredClient {
    client: JoinHandle<()>,
    sender: Sender<ClientMessage>,
}

/// this actor manager all of clients
pub struct ClientSupervisor {
    clients: HashMap<String, StoredClient>,
    inbox: Receiver<<Self as Dctor>::InboxItem>,
    sender: Sender<<Self as Dctor>::InboxItem>,
}

impl ClientSupervisor {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        ClientSupervisor {
            clients: HashMap::new(),
            inbox: rx,
            sender: tx,
        }
    }
}

impl Dctor for ClientSupervisor {
    type InboxItem = SupervisorMessage;

    fn listen(&mut self) {
        use SupervisorMessage::*;
        while let Ok(msg) = self.inbox.recv() {
            match msg {
                NewClient(username, tcp_stream) => {
                    let (mut client, client_sender) = Client::new(tcp_stream);

                    let handle = thread::spawn(move || {
                        client.listen();
                    });

                    self.clients.insert(
                        username,
                        StoredClient {
                            client: handle,
                            sender: client_sender,
                        },
                    );
                }
                Message {
                    sender,
                    receiver,
                    message,
                } => {
                    if !self.clients.contains_key(&sender) || !self.clients.contains_key(&receiver)
                    {
                        continue;
                    }

                    let receiver = self.clients.get_mut(&receiver).unwrap();
                    receiver
                        .sender
                        .send(ClientMessage::ReceiveMessage(sender, message))
                        .unwrap();
                }
                DisconnectClient(username) => {
                    if let Some(client) = self.clients.remove(&username) {
                        client.sender.send(ClientMessage::Terminate).unwrap();
                    }
                }
            }
        }
    }
}
