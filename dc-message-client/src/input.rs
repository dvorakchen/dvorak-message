use std::sync::Arc;

use super::client::{ClientMessage, ClientSender};
use tokio::io::{self, AsyncBufReadExt, BufReader};

///  the Actor that listen text input
pub(crate) struct Input {
    client_sender: ClientSender,
}

impl Input {
    pub fn new(client_sender: ClientSender) -> Self {
        Input { client_sender }
    }

    /// listen input from terminal,
    /// running into spread thread
    pub async fn listen(&self) {
        let client_sender = Arc::clone(&self.client_sender);
        let mut stdin = BufReader::new(io::stdin()).lines();

        tokio::spawn(async move {
            loop {
                if let Some(line) = stdin.next_line().await.unwrap() {
                    if let Ok(input) = InputType::parse(&line) {
                        match input {
                            InputType::Text(data) => {
                                client_sender.send(ClientMessage::Text(data)).await.unwrap();
                            }
                            InputType::Instruct(instruct) => match instruct {
                                Instruct::Quit => {
                                    client_sender.send(ClientMessage::Quit).await.unwrap();
                                    break;
                                }
                                Instruct::To(username) => {
                                    client_sender
                                        .send(ClientMessage::To(username))
                                        .await
                                        .unwrap();
                                }
                            },
                        }
                    }
                }
            }
        });
    }
}

/// the input line from IO
pub(crate) enum InputType {
    /// pure text, like message to client else
    Text(String),
    /// some special input, representing some special ability
    Instruct(Instruct),
}

pub(crate) enum Instruct {
    To(String),
    Quit,
}

impl Instruct {
    fn new(text: &str) -> Result<Instruct, String> {
        match text {
            "quit" => Ok(Instruct::Quit),
            t if t.starts_with("to:") => {
                let username = t[3..].trim();
                Ok(Instruct::To(username.to_string()))
            }
            _ => Err(format!("unknow text: {text}")),
        }
    }
}

impl InputType {
    /// parse line into Input
    pub fn parse(line: &str) -> Result<Self, String> {
        let line = line.trim();
        //  if input line start with '/' and not '//' there will be Instruct
        if line.starts_with("/") && !line.starts_with("//") {
            let line = &line[1..];
            let instruct = Instruct::new(line)?;
            Ok(InputType::Instruct(instruct))
        } else {
            let text_line = if line.starts_with("//") {
                &line[1..]
            } else {
                line
            };

            Ok(InputType::Text(text_line.to_string()))
        }
    }
}
