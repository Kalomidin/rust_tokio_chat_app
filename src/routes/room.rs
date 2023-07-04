use super::models::{CreateRoomRequest, RemoveUserRequest};

use crate::db::member::{count_active_members, create_new_member, delete_member, get_member};
use crate::db::room::{create_new_room, delete_room, get_room_by_id};
use crate::db::user::{get_user_by_id, get_user_by_name};

use crate::errors::ServiceError;
use crate::errors::{db_error_to_service_error, internal_error_to_service_error};
use crate::ws::lobby::{upgrade_to_websocket, Lobby};
use crate::ws::{ClientWsMessage, ClientWsMessageType};
use crate::ConnectionPool;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::Extension, extract::Path, extract::State, extract::WebSocketUpgrade, Json};
use std::sync::Arc;

pub async fn create_room(
  State(pool): State<ConnectionPool>,
  Extension(user_id): Extension<i64>,
  Json(create_room_request): Json<CreateRoomRequest>,
) -> Result<Json<serde_json::Value>, ServiceError> {
  let mut conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let room = create_new_room(&mut conn, create_room_request.name, user_id)
    .await
    .map_err(db_error_to_service_error)?;
  let member = create_new_member(&mut conn, room.id, user_id)
    .await
    .map_err(db_error_to_service_error)?;

  Ok(Json(serde_json::json!({
  "roomId": room.id,
  "roomName": room.name,
  "createdAt": room.created_at,
  "memberCreatedAt": member.created_at,
  "memberId": member.id,
  })))
}

pub async fn join_room(
  ws: WebSocketUpgrade,
  State(pool): State<ConnectionPool>,
  Extension(lobby): Extension<Arc<Lobby>>,
  Extension(user_id): Extension<i64>,
  Path(room_id): Path<i64>,
) -> Result<impl IntoResponse, ServiceError> {
  let mut conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let room = get_room_by_id(&mut conn, room_id)
    .await
    .map_err(db_error_to_service_error)?;
  if room.deleted_at != None {
    return Err(ServiceError::new(
      StatusCode::BAD_REQUEST,
      "Room is deleted",
    ));
  }
  let member = match get_member(&mut conn, room_id, user_id).await {
    Ok(member) => member,
    Err(_) => create_new_member(&mut conn, room.id, user_id)
      .await
      .map_err(db_error_to_service_error)?,
  };
  let user = get_user_by_id(&mut conn, user_id)
    .await
    .map_err(db_error_to_service_error)?;

  // Create web socket conn
  Ok(ws.on_upgrade(move |socket| {
    upgrade_to_websocket(socket, lobby, user_id, room, member, user.name)
  }))
}

pub async fn leave_room(
  State(pool): State<ConnectionPool>,
  Extension(user_id): Extension<i64>,
  Extension(lobby): Extension<Arc<Lobby>>,
  Path(room_id): Path<i64>,
) -> Result<Json<serde_json::Value>, ServiceError> {
  let mut conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let room = get_room_by_id(&mut conn, room_id)
    .await
    .map_err(db_error_to_service_error);
  if let Err(_) = room {
    return Err(ServiceError::new(
      StatusCode::BAD_REQUEST,
      "Room does not exist",
    ));
  }
  let room = room.unwrap();

  let deleted_member_id = delete_member(&mut conn, room.id, user_id)
    .await
    .map_err(db_error_to_service_error);
  if let Err(_) = deleted_member_id {
    return Err(ServiceError::new(
      StatusCode::BAD_REQUEST,
      "User is not a member of the room",
    ));
  }
  let deleted_member_id = deleted_member_id.unwrap();

  let active_member_count = count_active_members(&mut conn, room.id)
    .await
    .map_err(db_error_to_service_error)?;
  if active_member_count == 0 {
    delete_room(&mut conn, room_id)
      .await
      .map_err(db_error_to_service_error)?;
  }
  let user = get_user_by_id(&mut conn, user_id)
    .await
    .map_err(db_error_to_service_error)?;
  let user_name = user.name.clone();
  {
    let mut lobby_mutex = lobby.rooms.lock().unwrap();
    let room = lobby_mutex.get_mut(&room_id).unwrap();
    room
      .tx
      .send(
        serde_json::to_string(&ClientWsMessage {
          member_id: deleted_member_id,
          member_name: user_name,
          message_type: ClientWsMessageType::Leave,
          message: "left the room by user's request".to_owned(),
          db_skip_write: true,
        })
        .unwrap(),
      )
      .unwrap();
    println!("Sent leave message to user {:}", user_id);
  }

  Ok(Json(serde_json::json!({
  "roomName": room.name,
  "createdAt": room.created_at,
  "memberId": deleted_member_id,
  "activeMembersCount": active_member_count,
  })))
}

pub async fn remove_member(
  State(pool): State<ConnectionPool>,
  Extension(lobby): Extension<Arc<Lobby>>,
  Extension(user_id): Extension<i64>,
  Path(room_id): Path<i64>,
  Json(remove_user_request): Json<RemoveUserRequest>,
) -> Result<Json<serde_json::Value>, ServiceError> {
  let mut conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let room = get_room_by_id(&mut conn, room_id)
    .await
    .map_err(db_error_to_service_error);
  if let Err(_) = room {
    return Err(ServiceError::new(
      StatusCode::BAD_REQUEST,
      "Room does not exist",
    ));
  }
  let room = room.unwrap();

  if user_id != room.created_by {
    return Err(ServiceError::new(
      StatusCode::UNAUTHORIZED,
      "Unauthorized".to_string(),
    ));
  }
  let member_name = remove_user_request.user_name;
  let user = get_user_by_name(&mut conn, member_name)
    .await
    .map_err(db_error_to_service_error)?;

  let deleted_member_id = delete_member(&mut conn, room.id, user.id)
    .await
    .map_err(db_error_to_service_error);
  if let Err(_) = deleted_member_id {
    return Err(ServiceError::new(
      StatusCode::BAD_REQUEST,
      "User is not a member of the room",
    ));
  }
  let deleted_member_id = deleted_member_id.unwrap();

  let active_member_count = count_active_members(&mut conn, room.id)
    .await
    .map_err(db_error_to_service_error)?;
  if active_member_count == 0 {
    delete_room(&mut conn, room_id)
      .await
      .map_err(db_error_to_service_error)?;
  }

  let user_name = user.name.clone();
  {
    let mut lobby_mutex = lobby.rooms.lock().unwrap();
    match lobby_mutex.get_mut(&room_id) {
      Some(room) => {
        room
          .tx
          .send(
            serde_json::to_string(&ClientWsMessage {
              member_id: deleted_member_id,
              member_name: user_name,
              message_type: ClientWsMessageType::Leave,
              message: "user is kicked".to_owned(),
              db_skip_write: true,
            })
            .unwrap(),
          )
          .unwrap();
        println!("Sent kick message to user {:}", user_id);
      }
      _ => {
        // this means user created the room but noone joined it
      }
    }
  }

  Ok(Json(serde_json::json!({
  "roomName": room.name,
  "createdAt": room.created_at,
  "removedUserId": user.id,
  "removedUserName": user.name,
  "activeMembersCount": active_member_count,
  })))
}
