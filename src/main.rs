mod message;
mod server;

use std::time::Duration;

use message::Message;
use server::Server;
use tokio::io::AsyncBufReadExt;
use tokio::{io, signal, time};

#[tokio::main]
async fn main() {
    let mut gossip_interval = time::interval(Duration::from_millis(150));
    let mut server = Server::new();

    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin);
    let mut lines = reader.lines();

    loop {
        tokio::select! {
            _ = gossip_interval.tick() => {
                server.gossip();
            }
            line = lines.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        if let Ok(msg) = serde_json::from_str::<Message>(&line) {
                            server.handle(msg);
                        }
                    }
                    Ok(None) => {
                        server.shutdown();
                    }
                    Err(e) => eprintln!("Error reading line: {:?}", e),
                }
            }
            _ = server.shutdown.cancelled() => {
                break;
            }
            _ = signal::ctrl_c() => {
                server.shutdown();
            }
        }
    }

    eprintln!("Server shutting down");
    server.tracker.wait().await;
}
