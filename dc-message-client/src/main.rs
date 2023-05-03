use client::Client;
use tokio::net::TcpStream;

use clap::Parser;

use dvorak_message::message::{Message, MessageType};

mod input;
mod client;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    username: String,
}

#[tokio::main]
async fn main() {
    let arg = Args::parse();
    let username = arg.username.clone();

    //  连接服务器 ::8233
    let mut stream = TcpStream::connect("127.0.0.1:8233").await.unwrap();
    login(&mut stream, username.clone()).await;

    let mut client = Client::new(username, stream);

    let handler = tokio::spawn(async move {
        client.listen().await;
    });

    handler.await.unwrap();
}

async fn login(stream: &mut TcpStream, username: String) {
    Message::send(
        stream,
        Message::new(MessageType::Login, username, String::new()),
    )
    .await
    .unwrap();
}
