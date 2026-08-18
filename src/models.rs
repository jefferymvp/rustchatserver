use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEnvelope {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SayPayload {
    #[serde(default = "default_sender")]
    pub from: String,
    #[serde(default = "default_target")]
    pub to: String,
    #[serde(default)]
    pub msg: String,
}

fn default_sender() -> String {
    "匿名用户".to_string()
}

fn default_target() -> String {
    "all".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SayMsg {
    pub time: i64,
    pub data: SayPayload,
    pub notice: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMsg {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub msg: String,
    pub time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFlushMsg {
    pub users: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeMsg {
    pub msg: String,
    pub time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMsg {
    pub from: String,
    pub to: String,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadPayload {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileBuffer")]
    pub file_buffer: String,
}
