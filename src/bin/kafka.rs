use glomers::message::Service;
use glomers::server::{Context, Server};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Send {
        key: String,
        msg: u64,
    },
    SendOk {
        offset: u64,
    },
    Poll {
        offsets: HashMap<String, u64>,
    },
    PollOk {
        msgs: HashMap<String, Vec<[u64; 2]>>,
    },
    CommitOffsets {
        offsets: HashMap<String, u64>,
    },
    CommitOffsetsOk,
    ListCommittedOffsets {
        keys: Vec<String>,
    },
    ListCommittedOffsetsOk {
        offsets: HashMap<String, u64>,
    },
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Kafka {
    logs: HashMap<String, Vec<u64>>,
    commits: HashMap<String, u64>,
}

impl Service for Kafka {
    type Output = Message;
    fn handle(&mut self, msg: Message, _ctx: Context) -> Option<Message> {
        Some(match msg {
            Message::Send { key, msg } => {
                let entry = self.logs.entry(key).or_default();
                entry.push(msg);

                Message::SendOk {
                    offset: entry.len() as u64 - 1,
                }
            }
            Message::Poll { offsets } => {
                let mut msgs: HashMap<String, Vec<[u64; 2]>> = HashMap::new();
                offsets.iter().for_each(|(key, start_offset)| {
                    if let Some(logs) = self.logs.get(key) {
                        logs.iter().enumerate().for_each(|(i, msg)| {
                            if (i as u64) < *start_offset {
                                return;
                            }
                            msgs.entry(key.clone()).or_default().push([i as u64, *msg]);
                        });
                    }
                });
                Message::PollOk { msgs }
            }
            Message::CommitOffsets { offsets } => {
                offsets.into_iter().for_each(|(key, offset)| {
                    self.commits.entry(key).and_modify(|v| {
                        if *v < offset {
                            *v = offset;
                        }
                    });
                });
                Message::CommitOffsetsOk
            }
            Message::ListCommittedOffsets { keys } => {
                let mut offsets: HashMap<String, u64> = HashMap::new();
                keys.iter().for_each(|key| {
                    if let Some(offset) = self.commits.get(key) {
                        offsets.insert(key.clone(), *offset);
                    }
                });
                Message::ListCommittedOffsetsOk { offsets }
            }
            _ => unreachable!(),
        })
    }
}

#[tokio::main]
async fn main() {
    Server::<Kafka>::run(Default::default()).await;
}
