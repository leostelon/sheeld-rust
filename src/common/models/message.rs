use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    PING,
}

#[derive(Serialize, Deserialize)]
pub struct Message<T> {
    pub message_type: MessageType,
    pub data: T,
}