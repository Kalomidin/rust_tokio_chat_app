use super::models::CreateRoomRequest;

use crate::db::member::{create_new_member};
use crate::db::room::{create_new_room, get_room_by_id};

use crate::errors::ServiceError;
use crate::errors::{internal_error_to_service_error, db_error_to_service_error};
use crate::ConnectionPool;
use axum::{extract::Extension, extract::Path, extract::State, Json};

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
  "id": room.id,
  "roomName": room.name,
  "createdAt": room.created_at,
  "memberCreatedAt": member.created_at,
  "memberId": member.id,
  })))
}

pub async fn join_room(
  State(pool): State<ConnectionPool>,
  Extension(user_id): Extension<i64>,
  Path(room_id): Path<i64>,
) -> Result<Json<serde_json::Value>, ServiceError> {
  let mut _conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let room = get_room_by_id(&mut _conn, room_id)
    .await
    .map_err(db_error_to_service_error)?;
  let member = create_new_member(&mut _conn, room.id, user_id)
    .await
    .map_err(db_error_to_service_error)?;
  Ok(Json(serde_json::json!({
  "id": room.id,
  "roomName": room.name,
  "roomCreatedAt": room.created_at,
  "memberCreatedAt": member.created_at,
  "memberId": member.id,
  })))
}
