use std::sync::Arc;

use super::dctor::Dctor;
use super::supervisor::{SupervisorMessage, SupervisorSender};

use dvorak_message::message::{Message, MessageType};
use tokio::io::{stdin, AsyncBufReadExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};

use super::supervisor::ClientSupervisor;

/// representing the server,
/// listening the incoming client and io,
/// start and terminal the whole application
///
/// # example
/// ```
/// let server = Server::new("127.0.0.1:9998");
/// server.listen();
/// ```
pub struct Server {
    tcp_listener: TcpListener,
    supervisor_sender: SupervisorSender,
    sender: Arc<Sender<bool>>,
    inbox: Receiver<bool>,
}

impl Server {
    /// construct a Server
    pub async fn new(host: &str) -> Self {
        let tcp_listener = TcpListener::bind(host).await.unwrap();
        let (mut client_supervisor, supervisor_sender) = ClientSupervisor::new();
        let (tx, rx) = mpsc::channel(1);

        tokio::spawn(async move {
            client_supervisor.listen().await;
        });

        println!("Server construct.");
        Server {
            tcp_listener,
            supervisor_sender,
            sender: Arc::new(tx),
            inbox: rx,
        }
    }

    pub async fn listen(&mut self) {
        println!("Server Listening...");

        let input_handler = {
            let supervisor_sender = Arc::clone(&self.supervisor_sender);
            let server_sender = Arc::clone(&self.sender);
            tokio::spawn(async {
                Self::listen_input(supervisor_sender, server_sender).await;
            })
        };

        self.listen_incoming_client().await;

        input_handler.await.unwrap();
    }

    /// listen clients, and forward to supervisor
    async fn listen_incoming_client(&mut self) {
        loop {
            tokio::select! {
                tcp_message = self.tcp_listener.accept() => {
                    let (mut incoming_client, socket) = tcp_message.unwrap();

                    println!("Client incoming: {socket}");

                    let first = Server::check_login(&mut incoming_client).await;
                    if first.is_err() {
                        Message::send(
                            &mut incoming_client,
                            Message::new(
                                MessageType::Text("need login".to_string()),
                                "<Server>".to_string(),
                                String::new(),
                            ),
                        )
                        .await
                        .unwrap();
                        continue;
                    };
                    let username = first.unwrap();
                    println!("Client login success: {username}");

                    println!("Send message to supervisor");
                    self.supervisor_sender
                        .send(SupervisorMessage::NewClient(username, incoming_client))
                        .await
                        .unwrap();
                }
                is_quit = (self.inbox.recv()) => {
                    if let Some(true) = is_quit {
                        println!("Server received: QUIT, Server quit. Bye!");
                        break;
                    }
                }
            };
        }
    }

    /// if user type 'quit' in terminal, quit the application
    async fn listen_input(supervisor_sender: SupervisorSender, server_sender: Arc<Sender<bool>>) {
        let mut lines = BufReader::new(stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim() == "quit" {
                server_sender.send(true).await.unwrap();
                supervisor_sender
                    .send(SupervisorMessage::Terminate)
                    .await
                    .unwrap();
                return;
            }
        }
    }

    async fn check_login(tcp_stream: &mut TcpStream) -> Result<String, ()> {
        let message = Message::read_from(tcp_stream).await.unwrap();
        if message.is_none() {
            return Err(());
        }
        let message = message.unwrap();
        if message.message_type != MessageType::Login {
            return Err(());
        }

        Ok(message.username)
    }
}
