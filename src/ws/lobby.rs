use axum::extract::ws::{Message, WebSocket};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;

use super::{ClientWsMessage, ClientWsMessageType};
use tokio_postgres::NoTls;

use std::{
  collections::{HashMap, HashSet},
  sync::{Arc, Mutex},
};

use futures_util::{
  stream::{SplitSink, SplitStream, StreamExt},
  SinkExt,
};

use std::ops::ControlFlow;
use tokio::sync::broadcast;

use crate::db::member::{update_last_joined_at, Member};
use crate::db::message::add_message;
use crate::errors::internal_error_to_service_error;

use crate::{db::room::Room, errors::ServiceError};

pub struct Lobby {
  // We require unique usernames. This tracks which usernames have been taken.
  pub rooms: Mutex<HashMap<i64, RoomState>>,
  pub pool: Pool<PostgresConnectionManager<NoTls>>,
}

pub struct RoomState {
  // The set of all connected clients.
  pub clients: HashSet<i64>,

  // The name of the room.
  pub name: String,
  pub tx: broadcast::Sender<String>,
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
  member: Member,
  user_name: String,
) {
  // By splitting we can send and receive at the same time.
  let (sender, receiver) = stream.split();
  // We have more state now that needs to be pulled out of the connect loop
  let mut tx: Option<broadcast::Sender<String>> = None::<broadcast::Sender<String>>;
  let member_id = member.id;

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

  let rx_db = tx.subscribe();

  let mut sender_task = create_sender_task(sender, rx, member.clone());

  let mut db_write_task = create_db_write_task(room.id, rx_db, state.pool.clone());

  let mut receiver_task = create_receiver_task(receiver, tx.clone(), member, user_name.clone());

  // If any one of the tasks run to completion, we abort the other.
  tokio::select! {
      _ = (&mut sender_task) => {
        println!("sender task completed, member_id: {}, room_id: {}", member_id, room.id);
        receiver_task.abort();
        db_write_task.abort();
      },
      _ = (&mut receiver_task) => {
        println!("receiver task completed, member_id: {}, room_id: {}", member_id, room.id);
        sender_task.abort();
        db_write_task.abort();
      },
      _ = (&mut db_write_task) => {
        println!("db write task completed, member_id: {}, room_id: {}", member_id, room.id);
        sender_task.abort();
        receiver_task.abort();
      },
  };

  let room_id: i64 = room.id;
  remove_user_from_room(member_id, user_id, room_id, user_name, tx, state);
}

pub fn remove_user_from_room(
  member_id: i64,
  user_id: i64,
  room_id: i64,
  user_name: String,
  tx: broadcast::Sender<String>,
  state: Arc<Lobby>,
) {
  // send "user left" message
  let msg = ClientWsMessage {
    member_id,
    message_type: ClientWsMessageType::Message,
    member_name: user_name.clone(),
    message: format!("{} left the room", user_name),
  };
  tx.send(serde_json::to_string(&msg).unwrap()).unwrap();
  {
    // remove the user from the room
    let mut rooms = state.rooms.lock().unwrap();
    println!(">>> removing user from room: {}", room_id);
    let room_state = match rooms.get_mut(&room_id) {
      Some(room_state) => room_state,
      None => return,
    };
    room_state.clients.remove(&user_id);
    if room_state.clients.is_empty() {
      rooms.remove(&room_id);
    }
  }
  let pool = state.pool.clone();

  // update db status for member's table
  tokio::spawn(async move {
    let mut conn: bb8::PooledConnection<'_, PostgresConnectionManager<NoTls>> = pool
      .get()
      .await
      .map_err(internal_error_to_service_error)
      .unwrap();
    match update_last_joined_at(&mut conn, room_id, member_id).await {
      Ok(_) => {
        println!(">>> {} left the room", user_name);
      }
      Err(e) => {
        println!(
          "error updating last_joined_at for member: {}, err: {}",
          user_name, e
        );
        println!("member_id: {}, room_id: {}", member_id, room_id);
      }
    }
  });
}

fn create_db_write_task(
  room_id: i64,
  mut rx: broadcast::Receiver<String>,
  pool: bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>>,
) -> tokio::task::JoinHandle<Result<(), ServiceError>> {
  tokio::spawn(async move {
    while let Ok(msg) = rx.recv().await {
      match serde_json::from_str::<ClientWsMessage>(&msg) {
        Ok(m) => {
          let mut conn = pool.get().await.map_err(internal_error_to_service_error)?;
          match add_message(&mut conn, room_id, m.member_id, &m.message).await {
            Ok(_) => {
              println!(">>> {} sent msg: {:?} saved in db", m.member_id, m.message);
            }
            Err(e) => {
              println!(
                "error saving msg: {:?} from {} in to db, err: {:}",
                m.message, m.member_id, e
              );
            }
          }
        }
        _ => {
          println!("error parsing message");
        }
      }
    }
    Ok(())
  })
}

fn create_receiver_task(
  mut receiver: SplitStream<WebSocket>,
  tx: broadcast::Sender<String>,
  member: Member,
  member_name: String,
) -> tokio::task::JoinHandle<()> {
  tokio::spawn(async move {
    while let Some(msg) = receiver.next().await {
      // In any websocket error, break loop.
      // TODO: handle msg error
      if process_message(tx.clone(), msg.unwrap(), member_name.clone(), member.id).is_break() {
        break;
      }
    }
  })
}

fn create_sender_task(
  mut sender: SplitSink<WebSocket, Message>,
  mut rx: broadcast::Receiver<String>,
  member: Member,
) -> tokio::task::JoinHandle<()> {
  tokio::spawn(async move {
    while let Ok(msg) = rx.recv().await {
      match serde_json::from_str::<ClientWsMessage>(&msg) {
        Ok(m) => match m.message_type {
          ClientWsMessageType::Kick => {
            if m.member_id == member.id {
              let msg = Message::Text(m.message);
              if sender.send(msg).await.is_err() {
                break;
              }
              return;
            }
          }
          ClientWsMessageType::Message => {
            // In any websocket error, break loop.
            if member.id != m.member_id
              && sender
                .send(Message::Text(m.message.to_owned()))
                .await
                .is_err()
            {
              println!(
                "error sending message from {} to {}",
                m.member_id, member.id
              );
              break;
            } else {
              println!(
                ">>> {} sent str: {:?} to {}",
                m.member_id, m.message, member.id
              );
            }
          }
        },
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
  member_name: String,
  member_id: i64,
) -> ControlFlow<(), ()> {
  match msg {
    Message::Text(t) => {
      println!(">>> {} sent str: {:?}", member_id, t);
      tx.send(
        serde_json::to_string(&ClientWsMessage {
          member_id,
          member_name,
          message_type: ClientWsMessageType::Message,
          message: t,
        })
        .unwrap(),
      )
      .unwrap();
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
