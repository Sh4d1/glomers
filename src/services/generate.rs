use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateOk {
    id: String,
}

impl Default for GenerateOk {
    fn default() -> Self {
        Self {
            id: rusty_ulid::generate_ulid_string(),
        }
    }
}
