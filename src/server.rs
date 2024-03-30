use crate::message::{Body, Message, MessageType};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use tokio::{
    io::{self, AsyncWriteExt},
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub struct Server {
    tx: mpsc::UnboundedSender<Message>,
    node_id: String,
    node_ids: Vec<String>,
    neighbours: HashMap<String, HashSet<u64>>,
    values: HashSet<u64>,

    pub shutdown: CancellationToken,
    pub tracker: TaskTracker,
}

impl Server {
    pub fn shutdown(&mut self) {
        self.shutdown.cancel();
    }

    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        let shutdown = CancellationToken::new();
        let tracker = TaskTracker::new();

        {
            let shutdown = shutdown.clone();
            tracker.spawn(async move {
                let mut stdout = io::stdout();
                loop {
                    tokio::select! {
                        rcv = rx.recv() => match rcv {
                            Some(msg) => {
                                stdout
                                    .write_all(serde_json::to_string(&msg).unwrap().as_bytes())
                                    .await
                                    .unwrap();
                                stdout.write_all(b"\n").await.unwrap();
                                stdout.flush().await.unwrap();
                            },
                            None => {
                                shutdown.cancel();
                                return;
                            },
                        },
                        _ = shutdown.cancelled() => {
                                rx.close();
                        }
                    }
                }
            });
            tracker.close();
        }
        Self {
            tx,
            node_id: Default::default(),
            node_ids: Default::default(),
            neighbours: Default::default(),
            values: Default::default(),
            shutdown,
            tracker,
        }
    }

    pub async fn gossip(&mut self) {
        let values: Vec<u64> = self.values.iter().copied().collect();
        self.neighbours.iter().for_each(|(node_id, known)| {
            let values: Vec<u64> = values
                .iter()
                .filter(|v| !known.contains(v))
                .copied()
                .collect();
            if !values.is_empty() {
                self.send_gossip(node_id.clone(), values);
            }
        });
    }

    pub fn send_gossip(&self, node_id: String, values: Vec<u64>) {
        let msg = Message {
            src: self.node_id.clone(),
            dest: node_id,
            body: Body {
                message_type: MessageType::Gossip {
                    values: values.into_iter().collect(),
                },
                msg_id: None,
                in_reply_to: None,
            },
        };
        self.tx.send(msg).unwrap();
    }

    pub async fn handle(&mut self, msg: Message) -> Result<()> {
        let mut msg = msg;

        std::mem::swap(&mut msg.src, &mut msg.dest);

        if let Some(msg_id) = &mut msg.body.msg_id {
            msg.body.in_reply_to = Some(*msg_id);
        }

        let message_type = match msg.body.message_type {
            MessageType::Init { node_id, node_ids } => {
                self.node_id = node_id;
                self.node_ids = node_ids;
                self.node_ids.sort();

                let n = 5;

                let current_pos = self
                    .node_ids
                    .iter()
                    .position(|v| v == &self.node_id)
                    .expect("current node not in node list")
                    % n;

                self.neighbours = self
                    .node_ids
                    .iter()
                    .enumerate()
                    .filter_map(|(i, node)| {
                        if (i + 1) % n == current_pos {
                            Some((node.clone(), HashSet::new()))
                        } else {
                            None
                        }
                    })
                    .collect();

                eprintln!("neighbours: {:?}", self.neighbours);
                MessageType::InitOk
            }

            MessageType::Echo { echo } => MessageType::EchoOk { echo },

            MessageType::Generate => MessageType::GenerateOk {
                id: rusty_ulid::generate_ulid_string(),
            },

            MessageType::Broadcast { message } => {
                self.values.insert(message);
                MessageType::BroadcastOk
            }

            MessageType::Read => {
                let messages: Vec<u64> = self.values.iter().copied().collect();
                MessageType::ReadOk { messages }
            }

            MessageType::Topology { .. } => MessageType::TopologyOk,

            MessageType::Gossip { values } => {
                self.values.extend(values.clone());
                self.neighbours.entry(msg.dest).or_default().extend(values);
                return Ok(());
            }

            MessageType::InitOk
            | MessageType::EchoOk { .. }
            | MessageType::GenerateOk { .. }
            | MessageType::ReadOk { .. }
            | MessageType::BroadcastOk
            | MessageType::TopologyOk => return Ok(()),
        };

        msg.body.message_type = message_type;
        self.tx.send(msg)?;
        Ok(())
    }
}
