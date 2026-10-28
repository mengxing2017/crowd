use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub nickname: String,
    pub avatar: Option<String>,
    pub version: u64,
    pub status: String,
    pub roles: Vec<String>,
    pub scope: Vec<String>,
    pub receipts: Vec<Value>,
    pub restrictions: Vec<Value>,
    pub agreement: Value,
    pub source: Option<String>,
}
