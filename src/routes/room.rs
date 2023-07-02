use super::models::{CreateRoomRequest, RemoveUserRequest};

use crate::db::member::{create_new_member, delete_member, count_active_members};
use crate::db::room::{create_new_room, get_room_by_id, delete_room};
use crate::db::user::{get_user_by_name};

use crate::errors::ServiceError;
use crate::errors::{db_error_to_service_error, internal_error_to_service_error};
use crate::ws::lobby::{upgrade_to_websocket, Lobby};
use crate::ConnectionPool;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::Extension, extract::Path, extract::State, extract::WebSocketUpgrade, Json};
use std::{
  collections::HashSet,
  sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

pub async fn create_room(
  State(pool): State<ConnectionPool>,
  Extension(user_id): Extension<i64>,
  Json(create_room_request): Json<CreateRoomRequest>,
) -> Result<Json<serde_json::Value>, ServiceError> {
  let mut _conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let room = create_new_room(&mut _conn, create_room_request.name, user_id)
    .await
    .map_err(db_error_to_service_error)?;
  let member = create_new_member(&mut _conn, room.id, user_id)
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
  State(lobby): State<Arc<Lobby>>,
  Extension(user_id): Extension<i64>,
  Path(room_id): Path<i64>,
) -> Result<impl IntoResponse, ServiceError> {
  let mut _conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let room = get_room_by_id(&mut _conn, room_id)
    .await
    .map_err(db_error_to_service_error)?;
  if room.deleted_at != None {
    return Err(ServiceError::new(
      StatusCode::BAD_REQUEST,
      "Room is deleted",
    ));
  }
  create_new_member(&mut _conn, room.id, user_id)
    .await
    .map_err(db_error_to_service_error)?;

  // Create web socket conn
  Ok(ws.on_upgrade(move |socket| upgrade_to_websocket(socket, lobby, user_id)))
}

pub async fn leave_room(
  State(pool): State<ConnectionPool>,
  Extension(user_id): Extension<i64>,
  Path(room_id): Path<i64>,
) -> Result<Json<serde_json::Value>, ServiceError> {
  let mut conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let room = get_room_by_id(&mut conn, room_id)
    .await
    .map_err(db_error_to_service_error)?;
  let member = delete_member(&mut conn, room.id, user_id)
    .await
    .map_err(db_error_to_service_error)?;

  let active_member_count = count_active_members(&mut conn, room.id)
    .await
    .map_err(db_error_to_service_error)?;
  if active_member_count == 0 {
    delete_room(&mut conn, room_id).await.map_err(db_error_to_service_error)?;
  }
  
  Ok(Json(serde_json::json!({
    "roomName": room.name,
    "createdAt": room.created_at,
    "memberCreatedAt": member.created_at,
    "memberDeletedAt": member.deleted_at,
    "memberId": member.id,
    "activeMembersCount": active_member_count,
    })))
}

pub async fn remove_member(
  State(pool): State<ConnectionPool>,
  Extension(user_id): Extension<i64>,
  Path(room_id): Path<i64>,
  Json(remove_user_request): Json<RemoveUserRequest>,
) -> Result<Json<serde_json::Value>, ServiceError> {
  let mut conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let room = get_room_by_id(&mut conn, room_id)
    .await
    .map_err(db_error_to_service_error)?;
  if user_id != room.created_by {
    return Err(ServiceError::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
  }
  let member_name = remove_user_request.user_name;
  let user = get_user_by_name(&mut conn, member_name)
    .await
    .map_err(db_error_to_service_error)?;


  delete_member(&mut conn, room.id, user.id)
    .await
    .map_err(db_error_to_service_error)?;

  let active_member_count = count_active_members(&mut conn, room.id)
    .await
    .map_err(db_error_to_service_error)?;
  if active_member_count == 0 {
    delete_room(&mut conn, room_id).await.map_err(db_error_to_service_error)?;
  }
  
  Ok(Json(serde_json::json!({
    "roomName": room.name,
    "createdAt": room.created_at,
    "removedUserId": user.id,
    "removedUserName": user.name,
    "activeMembersCount": active_member_count,
    })))
}
