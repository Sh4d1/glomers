use glomers::message::Service;
use glomers::server::{Context, Server};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Generate,
    GenerateOk { id: String },
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Generate {}

impl Service for Generate {
    type Output = Message;
    fn handle(&mut self, msg: Message, _: Context) -> Option<Message> {
        match msg {
            Message::Generate => Some(Message::GenerateOk {
                id: rusty_ulid::generate_ulid_string(),
            }),
            _ => unreachable!(),
        }
    }
}

#[tokio::main]
async fn main() {
    Server::<Generate>::run(Default::default()).await;
}
