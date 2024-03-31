use glomers::message::Service;
use glomers::server::{Context, Runtime, Server};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Broadcast {
        message: u64,
    },
    BroadcastOk,
    Gossip {
        values: HashSet<u64>,
    },
    Read,
    ReadOk {
        messages: Vec<u64>,
    },
    Topology {
        topology: HashMap<String, HashSet<String>>,
    },
    TopologyOk,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Broadcast {
    values: HashSet<u64>,
    neighbours: HashSet<String>,
}

impl Service for Broadcast {
    type Output = Message;
    fn handle(&mut self, msg: Message, ctx: Context) -> Option<Message> {
        Some(match msg {
            Message::Broadcast { message } => {
                self.values.insert(message);
                Message::BroadcastOk
            }
            Message::Read => Message::ReadOk {
                messages: self.values.iter().copied().collect(),
            },
            Message::Topology { .. } => {
                // star topology
                self.neighbours = if ctx.rt.node_id == ctx.rt.node_ids[0] {
                    ctx.rt.node_ids.iter().skip(1).cloned().collect()
                } else {
                    [ctx.rt.node_ids[0].clone()].into()
                };
                Message::TopologyOk
            }

            Message::Gossip { values } => {
                self.values.extend(values.clone());
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
        let values: HashSet<u64> = self.values.iter().copied().collect();

        self.neighbours.iter().for_each(|node_id| {
            if !values.is_empty() {
                rt.send_msg::<Self>(
                    node_id,
                    Message::Gossip {
                        values: values.clone(),
                    },
                );
            }
        });
    }
}

#[tokio::main]
async fn main() {
    Server::<Broadcast>::run(Default::default()).await;
}
