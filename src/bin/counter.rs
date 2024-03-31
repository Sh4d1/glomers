use glomers::message::Service;
use glomers::server::{Context, Runtime, Server};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Gossip { counter: u64 },
    Add { delta: u64 },
    AddOk,
    Read,
    ReadOk { value: u64 },
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Counter {
    counter: u64,
    known_counter: HashMap<String, u64>,
}

impl Service for Counter {
    type Output = Message;
    fn handle(&mut self, msg: Message, ctx: Context) -> Option<Message> {
        Some(match msg {
            Message::Add { delta } => {
                self.counter += delta;
                Message::AddOk
            }
            Message::Read => Message::ReadOk {
                value: self.counter + self.known_counter.values().sum::<u64>(),
            },

            Message::Gossip { counter } => {
                self.known_counter.insert(ctx.msg.dest.clone(), counter);
                return None;
            }

            _ => unreachable!(),
        })
    }

    fn gossip_interval(&self) -> time::Interval {
        time::interval(Duration::from_millis(150))
    }

    fn gossip(&mut self, rt: &Runtime) {
        let rt = rt.clone();

        rt.node_ids.iter().for_each(|node_id| {
            if self.counter > 0 {
                rt.send_msg::<Self>(
                    node_id,
                    Message::Gossip {
                        counter: self.counter,
                    },
                );
            }
        });
    }
}

#[tokio::main]
async fn main() {
    Server::<Counter>::run(Default::default()).await;
}
