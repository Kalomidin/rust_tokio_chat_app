pub mod lobby;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClientWsMessage {
  pub member_id: i64,
  pub member_name: String,
  pub message_type: ClientWsMessageType,
  pub message: String,
  pub db_skip_write: bool,
}

#[derive(Serialize, Deserialize)]
pub enum ClientWsMessageType {
  Message,
  Leave,
}

pub enum ServerTaskTerminationReason {
  ClientDisconnected,
  ClientLeft,
}
