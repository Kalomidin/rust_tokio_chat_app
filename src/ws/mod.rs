pub mod lobby;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClientWsMessage {
  pub member_id: i64,
  pub member_name: String,
  pub message: String,
}
