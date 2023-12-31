use crate::ws::lobby::Lobby;
use crate::ConnectionPool;
use std::sync::Arc;

pub mod models;
pub mod room;
pub mod user;

pub struct SharedState {
  pub pool: ConnectionPool,
  pub lobby: Arc<Lobby>,
}
