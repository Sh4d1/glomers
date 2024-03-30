mod message;
mod server;
mod services;

use message::Message;
use server::Server;
use tokio::io::AsyncBufReadExt;
use tokio::io::{self, AsyncWriteExt};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin);
    let mut lines = reader.lines();

    let server = Server::new();

    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    tokio::spawn(async move {
        let mut stdout = io::stdout();
        while let Some(msg) = rx.recv().await {
            stdout
                .write_all(serde_json::to_string(&msg).unwrap().as_bytes())
                .await
                .unwrap();
            stdout.write_all(b"\n").await.unwrap();
            stdout.flush().await.unwrap();
        }
    });

    while let Some(line) = lines.next_line().await.unwrap() {
        let server = server.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            match serde_json::from_str::<Message>(&line) {
                Ok(msg) => match server.handle(msg, tx).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error handling echo message: {:?}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Error: {:?} : {:?}", e, line);
                }
            }
        });
    }
}
