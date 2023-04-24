use async_trait::async_trait;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use crate::client::Client;

use super::client::ClientMessage;
use super::dctor::Dctor;
use std::{collections::HashMap, sync::Arc};

pub type SupervisorSender = Arc<Sender<SupervisorMessage>>;

/// Actor Message for ClientSupervisor
#[derive(Debug)]
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
    /// close all of clients, and this Supervisor
    Terminate,
}

struct StoredClient {
    client: JoinHandle<()>,
    sender: Sender<ClientMessage>,
}

/// this actor manager all of clients
pub struct ClientSupervisor {
    clients: HashMap<String, StoredClient>,
    inbox: Receiver<<Self as Dctor>::InboxItem>,
    /// should keep a supervisor sender, for distribute to all clients
    sender: SupervisorSender,
}

impl ClientSupervisor {
    pub fn new() -> (Self, SupervisorSender) {
        let (tx, rx) = mpsc::channel(100);
        let supervisor_sender = Arc::new(tx);

        println!("Supervisor construct");
        (
            ClientSupervisor {
                clients: HashMap::new(),
                inbox: rx,
                sender: Arc::clone(&supervisor_sender),
            },
            supervisor_sender,
        )
    }
}

#[async_trait]
impl Dctor for ClientSupervisor {
    type InboxItem = SupervisorMessage;

    async fn listen(&mut self) {
        use SupervisorMessage::*;

        println!("Supervisor listening...");
        'listen: while let Some(msg) = self.inbox.recv().await {
            println!("Supervisor received message: {:?}", msg);
            match msg {
                NewClient(username, tcp_stream) => {
                    let (mut client, client_sender) =
                        Client::new(tcp_stream, Arc::clone(&self.sender));

                    let handle = tokio::spawn(async move {
                        client.listen().await;
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
                        .await
                        .unwrap();
                }
                DisconnectClient(username) => {
                    if let Some(client) = self.clients.remove(&username) {
                        client.sender.send(ClientMessage::Terminate).await.unwrap();
                    }
                }
                Terminate => {
                    for (_, stored_client) in self.clients.iter_mut() {
                        stored_client
                            .sender
                            .send(ClientMessage::Terminate)
                            .await
                            .unwrap();
                    }
                    self.clients.clear();
                    break 'listen;
                }
            }
        }
    }
}
