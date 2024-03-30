use serde::{Deserialize, Serialize};

use crate::services::{echo::Echo, generate::GenerateOk};

#[derive(Debug, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageType {
    #[serde(rename = "init")]
    Init(Init),
    #[serde(rename = "init_ok")]
    InitOk,

    #[serde(rename = "echo")]
    Echo(Echo),
    #[serde(rename = "echo_ok")]
    EchoOk(Echo),

    #[serde(rename = "generate")]
    Generate,
    #[serde(rename = "generate_ok")]
    GenerateOk(GenerateOk),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<u64>,
    #[serde(flatten)]
    pub message_type: MessageType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}
