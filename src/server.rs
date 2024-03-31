use crate::message::{self, to_extra, BodyType, Init, Message, Service};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt},
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub struct Context {
    pub rt: Runtime,
    pub msg: Message,
}

impl Context {
    pub fn new(rt: &Runtime, msg: Message) -> Self {
        Self {
            rt: rt.clone(),
            msg,
        }
    }
}

#[derive(Clone)]
pub struct Runtime {
    pub node_id: String,
    pub node_ids: Vec<String>,
    pub tx: mpsc::UnboundedSender<Message>,
}

impl Runtime {
    pub fn send_msg<S: Service>(&self, node_id: &String, msg: S::Output) {
        if node_id == &self.node_id {
            return;
        }
        drop(self.tx.send(message::Message {
            src: self.node_id.clone(),
            dest: node_id.clone(),
            body: message::Body {
                extra: to_extra(msg).expect("failed to serialize"),
                msg_id: None,
                in_reply_to: None,
            },
        }))
    }
}

pub struct Server<S>
where
    S: Service,
    <S as Service>::Output: serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    runtime: Runtime,
    service: S,

    pub shutdown: CancellationToken,
}

impl<S> Server<S>
where
    S: Service + 'static,
    <S as Service>::Output: serde::Serialize + for<'a> serde::Deserialize<'a> + Send,
{
    pub fn shutdown(&mut self) {
        self.shutdown.cancel();
    }

    pub async fn run(s: S) {
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

        let stdin = io::stdin();
        let reader = io::BufReader::new(stdin);
        let mut lines = reader.lines();
        let mut server = Self {
            service: s,
            shutdown,
            runtime: Runtime {
                tx,
                node_id: Default::default(),
                node_ids: Default::default(),
            },
        };

        let mut gossip_interval = server.service.gossip_interval();

        loop {
            tokio::select! {
                _ = gossip_interval.tick() => {
                    server.service.gossip(&server.runtime);
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
            }
        }
        tracker.wait().await;
    }

    pub fn handle(&mut self, msg: Message) {
        let mut msg = msg;

        std::mem::swap(&mut msg.src, &mut msg.dest);

        if let Some(msg_id) = &mut msg.body.msg_id {
            msg.body.in_reply_to = Some(*msg_id);
        }
        let ctx: Context = Context::new(&self.runtime, msg.clone());
        if let Ok(body) = msg.body.as_obj::<BodyType<S>>() {
            let extra = match body {
                BodyType::Init(Init::Init { node_id, node_ids }) => {
                    self.runtime.node_id = node_id;
                    self.runtime.node_ids = node_ids;
                    self.runtime.node_ids.sort();
                    to_extra(Init::InitOk).expect("failed to serialize")
                }
                BodyType::Rpc(_) => todo!(),
                BodyType::Service(msg) => match self.service.handle(msg, ctx) {
                    Some(msg) => to_extra(msg).expect("failed to serialize"),
                    None => return,
                },
                _ => unreachable!(),
            };
            msg.body.extra = extra;
            _ = self.runtime.tx.send(msg);
        }
    }
}
