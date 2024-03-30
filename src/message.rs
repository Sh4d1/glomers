use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,

    // echo
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },

    // unique ID
    Generate,
    GenerateOk {
        id: String,
    },

    // broadcast
    Broadcast {
        message: u64,
    },
    BroadcastOk,

    Gossip {
        values: HashSet<u64>,
        counter: u64,
    },

    // this should be read, but read is used for the grow only
    BrdRead,
    BrdReadOk {
        messages: Vec<u64>,
    },

    Topology {
        topology: HashMap<String, HashSet<String>>,
    },
    TopologyOk,

    // grow only counter
    Add {
        delta: u64,
    },
    AddOk,
    Read,
    ReadOk {
        value: u64,
    },
    // kafka style log
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Body {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<u64>,
    #[serde(flatten)]
    pub message_type: MessageType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}
