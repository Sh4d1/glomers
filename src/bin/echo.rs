use glomers::message::Service;
use glomers::server::{Context, Server};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Echo { echo: String },
    EchoOk { echo: String },
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Echo {}

impl Service for Echo {
    type Output = Message;
    fn handle(&mut self, msg: Message, _: Context) -> Option<Message> {
        match msg {
            Message::Echo { echo } => Some(Message::EchoOk { echo }),
            _ => unreachable!(),
        }
    }
}

#[tokio::main]
async fn main() {
    Server::<Echo>::run(Default::default()).await;
}
