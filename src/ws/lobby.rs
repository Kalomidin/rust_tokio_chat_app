use axum::{
  extract::ws::{Message, WebSocket, WebSocketUpgrade},
  response::IntoResponse,
  routing::get,
  Router,
};
use bb8::{ManageConnection, Pool};
use bb8_postgres::PostgresConnectionManager;

use super::ClientWsMessage;
use tokio_postgres::{config::Config, NoTls};
use tokio_postgres_migration::Migration;

use std::{
  collections::{HashMap, HashSet},
  sync::{Arc, Mutex},
};

use futures_util::{
  stream::{SplitSink, SplitStream, StreamExt},
  SinkExt,
};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use tokio::sync::broadcast;

use crate::db::room::Room;

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
  pub fn new(pool: Pool<PostgresConnectionManager<NoTls>>) -> Lobby {
    Lobby {
      rooms: Mutex::new(HashMap::new()),
      pool,
    }
  }
}

impl RoomState {
  pub fn new(name: String, tx: broadcast::Sender<String>) -> RoomState {
    RoomState {
      clients: HashSet::new(),
      name,
      tx,
    }
  }
}

pub async fn upgrade_to_websocket(
  stream: WebSocket,
  state: Arc<Lobby>,
  user_id: i64,
  room: Room,
  member_id: i64,
) {
  // By splitting we can send and receive at the same time.
  let (sender, receiver) = stream.split();
  // We have more state now that needs to be pulled out of the connect loop
  let mut tx: Option<broadcast::Sender<String>> = None::<broadcast::Sender<String>>;

  {
    // create or get the room state
    let room_id = room.id;
    let name = room.name;
    let mut rooms = state.rooms.lock().unwrap();
    let room_state = rooms.entry(room_id).or_insert_with(|| {
      let (tx, _) = broadcast::channel(10);
      RoomState::new(name, tx)
    });
    if !room_state.clients.contains(&user_id) {
      room_state.clients.insert(user_id);
    }
    tx = Some(room_state.tx.clone());
  }

  let tx = tx.unwrap();

  let rx = tx.subscribe();

  let mut sender_task = create_sender_task(sender, rx, member_id);

  let mut receiver_task = create_receiver_task(receiver, tx, member_id);

  // If any one of the tasks run to completion, we abort the other.
  tokio::select! {
      _ = (&mut sender_task) => receiver_task.abort(),
      _ = (&mut receiver_task) => sender_task.abort(),
  };

  // send "user left" message

  {
    // remove the user from the room
    let mut rooms = state.rooms.lock().unwrap();
    let room_state = rooms.get_mut(&room.id).unwrap();
    room_state.clients.remove(&user_id);
    if room_state.clients.is_empty() {
      rooms.remove(&room.id);
    }
  }

  // update db status for member's table
}

fn create_receiver_task(
  mut receiver: SplitStream<WebSocket>,
  tx: broadcast::Sender<String>,
  member_id: i64,
) -> tokio::task::JoinHandle<()> {
  tokio::spawn(async move {
    while let Some(msg) = receiver.next().await {
      // In any websocket error, break loop.
      // TODO: handle msg error
      if process_message(tx.clone(), msg.unwrap(), member_id).is_break() {
        break;
      }
    }
  })
}

fn create_sender_task(
  mut sender: SplitSink<WebSocket, Message>,
  mut rx: broadcast::Receiver<String>,
  member_id: i64,
) -> tokio::task::JoinHandle<()> {
  tokio::spawn(async move {
    while let Ok(msg) = rx.recv().await {
      match serde_json::from_str::<ClientWsMessage>(&msg) {
        Ok(m) => {
          // In any websocket error, break loop.
          if member_id != m.member_id
            && sender
              .send(Message::Text(m.message.to_owned()))
              .await
              .is_err()
          {
            println!(
              "error sending message from {} to {}",
              m.member_id, member_id
            );
            break;
          } else {
            println!(
              ">>> {} sent str: {:?} to {}",
              m.member_id, m.message, member_id
            );
          }
        }
        _ => {
          println!("error parsing message");
        }
      }
    }
  })
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
fn process_message(
  tx: broadcast::Sender<String>,
  msg: Message,
  member_id: i64,
) -> ControlFlow<(), ()> {
  match msg {
    Message::Text(t) => {
      println!(">>> {} sent str: {:?}", member_id, t);
      tx.send(
        serde_json::to_string(&ClientWsMessage {
          member_id,
          message: t,
        })
        .unwrap(),
      )
      .unwrap();

      // TODO: add into message table
    }
    Message::Binary(d) => {
      println!(">>> {} sent {} bytes: {:?}", member_id, d.len(), d);
    }
    Message::Close(c) => {
      if let Some(cf) = c {
        println!(
          ">>> {} sent close with code {} and reason `{}`",
          member_id, cf.code, cf.reason
        );
      } else {
        println!(
          ">>> {} somehow sent close message without CloseFrame",
          member_id
        );
      }
      return ControlFlow::Break(());
    }

    Message::Pong(v) => {
      println!(">>> {} sent pong with {:?}", member_id, v);
    }
    // You should never need to manually handle Message::Ping, as axum's websocket library
    // will do so for you automagically by replying with Pong and copying the v according to
    // spec. But if you need the contents of the pings you can see them here.
    Message::Ping(v) => {
      println!(">>> {} sent ping with {:?}", member_id, v);
    }
  }
  ControlFlow::Continue(())
}
