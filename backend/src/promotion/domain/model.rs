use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub struct Code {
    pub owner: String,
    pub version: u64,
    pub active: bool,
    pub product: Option<String>,
    pub task: Option<String>,
}
