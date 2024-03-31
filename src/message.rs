use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::time;

use crate::server::{Context, Runtime};

pub trait Service {
    type Output: Serialize + for<'a> Deserialize<'a>;

    fn handle(&mut self, msg: Self::Output, ctx: Context) -> Option<Self::Output>;
    fn gossip(&mut self, _: &Runtime) {}
    fn gossip_interval(&self) -> time::Interval {
        time::interval(time::Duration::from_secs(1))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Rpc {
    Read { key: String },
    ReadOk { value: Option<String> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Init {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum BodyType<S>
where
    S: Service,
{
    #[serde(with = "S::Output")]
    Service(S::Output),
    Rpc(Rpc),
    Init(Init),
}

pub fn to_extra<T>(t: T) -> anyhow::Result<Map<String, Value>>
where
    T: Serialize,
{
    match serde_json::to_value(t) {
        Ok(value) => match value {
            Value::Object(map) => Ok(map),
            _ => Err(anyhow!("expected object, got: {:?}", value)),
        },
        Err(e) => Err(anyhow!("failed to serialize: {}", e)),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Body {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<u64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Body {
    pub fn as_obj<'de, T>(&self) -> anyhow::Result<T>
    where
        T: Deserialize<'de>,
    {
        match T::deserialize(Value::Object(self.extra.clone())) {
            Ok(t) => Ok(t),
            Err(e) => Err(anyhow!("failed to deserialize: {}", e)),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    #[serde(with = "Body")]
    pub body: Body,
}
