use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Echo {
    echo: String,
}

impl Echo {
    pub fn reply(&self) -> Self {
        Self {
            echo: self.echo.clone(),
        }
    }
}
