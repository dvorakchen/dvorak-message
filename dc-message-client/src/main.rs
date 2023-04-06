use tokio::io::BufReader;
use tokio::io::{self, AsyncBufReadExt};
use tokio::net::TcpStream;
use tokio::sync::oneshot::{self, Receiver, Sender};

use clap::Parser;

use dvorak_message::message::{Message, MessageType};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    username: String,
    #[arg(short, long)]
    to: String,
}

#[tokio::main]
async fn main() {
    let arg = Args::parse();
    let username = arg.username.clone();
    let receiver = arg.to.clone();

    //  连接服务器 ::8233
    let mut stream = TcpStream::connect("127.0.0.1:8233").await.unwrap();
    login(&mut stream, username.clone()).await;

    let (mut read_stream, mut write_stream) = stream.into_split();

    let (tx, rx) = oneshot::channel::<bool>();

    let write_join = tokio::spawn(async move {
        read_and_send(&mut write_stream, username, tx, receiver).await;
    });

    let read_join = tokio::spawn(async move {
        receive(&mut read_stream, rx).await;
    });

    write_join.await.unwrap();
    read_join.await.unwrap();
}

async fn login(stream: &mut TcpStream, username: String) {
    Message::send(
        stream,
        Message::new(MessageType::Login, username, String::new()),
    )
    .await
    .unwrap();
}

async fn read_and_send(
    write_stream: &mut OwnedWriteHalf,
    username: String,
    tx: Sender<bool>,
    receiver: String,
) {
    let stdin = BufReader::new(io::stdin());
    let mut stdin = stdin.lines();

    loop {
        if let Some(line) = stdin.next_line().await.unwrap() {
            //  发送给服务器
            let line = line.trim().to_string();
            if line == "quit" {
                tx.send(true).unwrap();
                break;
            }

            let message = Message::new(MessageType::Text(line), username.clone(), receiver.clone());

            Message::send(write_stream, message).await.unwrap();
        }
    }
}

async fn receive(read_stream: &mut OwnedReadHalf, mut rx: Receiver<bool>) {
    loop {
        tokio::select! {
            is_quit = (&mut rx) => {
                if is_quit.unwrap() {
                    break;
                }
            }
            message = async {
                Message::read_from(read_stream).await.unwrap()

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
