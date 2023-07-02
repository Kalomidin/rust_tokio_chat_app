use axum::{
  extract::ws::{Message, WebSocket, WebSocketUpgrade},
  response::IntoResponse,
  routing::get,
  Router,
};
use bb8::{ManageConnection, Pool};
use bb8_postgres::PostgresConnectionManager;

use tokio_postgres::{config::Config, NoTls};
use tokio_postgres_migration::Migration;

use std::{
  collections::{HashMap, HashSet},
  sync::{Arc, Mutex},
};

use futures_util::stream::StreamExt;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use tokio::sync::broadcast;

pub struct Lobby {
  // We require unique usernames. This tracks which usernames have been taken.
  pub rooms: Mutex<HashMap<i64, RoomState>>,
  pub pool: Pool<PostgresConnectionManager<NoTls>>,
}

pub struct RoomState {
  // The set of all connected clients.
  clients: HashSet<i64>,

  // The name of the room.
  name: String,
  tx: broadcast::Sender<String>,
}


impl Lobby {
    pub fn new(pool:  Pool<PostgresConnectionManager<NoTls>>) -> Lobby {
      Lobby {
        rooms: Mutex::new(HashMap::new()),
        pool,
      }
    }
  }
  

pub async fn upgrade_to_websocket(stream: WebSocket, state: Arc<Lobby>, user_id: i64) {
  // By splitting we can send and receive at the same time.
  let (mut sender, mut receiver) = stream.split();
  // We have more state now that needs to be pulled out of the connect loop
  let mut tx = None::<broadcast::Sender<String>>;
  let mut username = String::new();
  let mut channel = String::new();
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
  match msg {
    Message::Text(t) => {
      println!(">>> {} sent str: {:?}", who, t);
    }
    Message::Binary(d) => {
      println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
    }
    Message::Close(c) => {
      if let Some(cf) = c {
        println!(
          ">>> {} sent close with code {} and reason `{}`",
          who, cf.code, cf.reason
        );
      } else {
        println!(">>> {} somehow sent close message without CloseFrame", who);
      }
      return ControlFlow::Break(());
    }

    Message::Pong(v) => {
      println!(">>> {} sent pong with {:?}", who, v);
    }
    // You should never need to manually handle Message::Ping, as axum's websocket library
    // will do so for you automagically by replying with Pong and copying the v according to
    // spec. But if you need the contents of the pings you can see them here.
    Message::Ping(v) => {
      println!(">>> {} sent ping with {:?}", who, v);
    }
  }
  ControlFlow::Continue(())
}
