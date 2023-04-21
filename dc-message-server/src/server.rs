use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

use dvorak_message::message::{Message, MessageType};

type ClientStore = Arc<RwLock<HashMap<String, TcpStream>>>;

pub async fn get_mut_ref<'a>(
    store: &'a ClientStore,
    username: &String,
) -> Option<&'a mut TcpStream> {
    store
        .write()
        .await
        .get_mut(username)
        .map(|t| unsafe { &mut *(t as *mut TcpStream) })
}

pub(crate) struct Server {
    keeping_clients: ClientStore,
    listener: TcpListener,
}

impl Server {
    pub async fn new(host: &str) -> Self {
        let listener = TcpListener::bind(host).await.unwrap();
        Self {
            keeping_clients: Arc::new(RwLock::new(HashMap::new())),
            listener,
        }
    }

    pub async fn listen(&mut self) {
        loop {
            let (mut client_stream, _) = self.listener.accept().await.unwrap();

            let first = Server::check_login(&mut client_stream).await;
            if first.is_err() {
                Message::send(
                    &mut client_stream,
                    Message::new(
                        MessageType::Text("need login".to_string()),
                        "<Server>".to_string(),
                        String::new(),
                    ),
                )
                .await
                .unwrap();
                continue;
            }

            let username = first.unwrap();
            println!("{username} connecting");
            self.keeping_clients
                .write()
                .await
                .insert(username.clone(), client_stream);
            let store = Arc::clone(&self.keeping_clients);

            tokio::spawn(async move {
                let result = Server::listen_client(username, store).await;
                if let Err(e) = result {
                    println!("{e}");
                }
            });
        }
    }

    async fn listen_client(username: String, store: ClientStore) -> Result<(), &'static str> {
        loop {
            let client_stream = get_mut_ref(&store, &username)
                .await
                .ok_or_else(|| "cannot find TcpStream")?;

            let message = Message::read_from(client_stream)
                .await
                .map_err(|_| "read from client failed")?
                .ok_or_else(|| "read from client failed")?;

            match &message.message_type {
                MessageType::Text(_) => {
                    println!("Received type: Text");
                    let receiver = &message.receiver.clone();

                    let receiver_stream = get_mut_ref(&store, &receiver).await;
                    if receiver_stream.is_none() {
                        println!("{} offline!", receiver);
                        let offline_message = Message::new(
                            MessageType::Text(format!("{} offline!", receiver)),
                            "<Server>".to_string(),
                            username.clone(),
                        );

                        Message::send(client_stream, offline_message).await.unwrap();
                        continue;
                    }

                    let receiver_stream = receiver_stream.unwrap();
                    Message::send(receiver_stream, message).await.unwrap();
                }
                MessageType::Logout => {
                    println!("Received type: Logout");
                    let mut hm = store.write().await;
                    hm.remove(&username);
                    println!("{} logged out!", username);
                    break;
                }
                _ => {
                    println!("Received type: other");
                    continue;
                }
            }
        }
        Ok(())
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
