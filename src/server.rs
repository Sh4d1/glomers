use anyhow::Result;

use crate::message::{Message, MessageType};

#[derive(Clone)]
pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn handle(
        &self,
        msg: Message,
        send_tx: tokio::sync::mpsc::UnboundedSender<Message>,
    ) -> Result<()> {
        let mut msg = msg;

        if let Some(msg_id) = &mut msg.body.msg_id {
            msg.body.in_reply_to = Some(*msg_id);
        }

        let message_type = match msg.body.message_type {
            MessageType::Echo(echo) => MessageType::EchoOk(echo.reply()),
            MessageType::EchoOk(_) => return Ok(()),
            MessageType::Init(_) => {
                msg.body.msg_id = None;
                MessageType::InitOk
            }
            MessageType::InitOk | MessageType::GenerateOk(_) => unreachable!(),
            MessageType::Generate => MessageType::GenerateOk(Default::default()),
        };
        msg.body.message_type = message_type;

        std::mem::swap(&mut msg.src, &mut msg.dest);

        send_tx.send(msg)?;
        Ok(())
    }
}
