pub mod models;
pub mod room;
pub mod user;

use crate::db::member::create_new_member;
use crate::db::room::{create_new_room, get_room_by_id};

use crate::errors::ServiceError;
use crate::errors::{db_error_to_service_error, internal_error_to_service_error};
use crate::ws::lobby::{upgrade_to_websocket, Lobby};
use crate::ConnectionPool;
use axum::response::IntoResponse;
use axum::{extract::Extension, extract::Path, extract::State, extract::WebSocketUpgrade, Json};
use std::{
  collections::HashSet,
  sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

pub struct SharedState {
  pub pool: ConnectionPool,
  pub lobby: Arc<Lobby>,
}
