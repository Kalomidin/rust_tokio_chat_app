use super::models::{NewUserRequest, LoginRequest};
use crate::db::user::{insert_new_user, get_user as get_user_from_db, get_user_by_id};
use crate::{internal_error, ConnectionPool};
use axum::{extract::State, http::StatusCode, Json, extract::Extension};
use crate::auth::create_jwt;

pub async fn signup(
  State(pool): State<ConnectionPool>,
  Json(user): Json<NewUserRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
  let _conn = pool.get().await.map_err(internal_error)?;
  let user = insert_new_user(_conn, user.name, user.password)
    .await
    .map_err(internal_error)?;
  Ok(Json(serde_json::json!({
    "id": user.id,
    "name": user.name,
    "createdAt": user.created_at,
    "authToken": create_jwt(user.id).unwrap()
  })))
}


pub async fn login(
  State(pool): State<ConnectionPool>,
  Json(user): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
  let _conn = pool.get().await.map_err(internal_error)?;
  let user = get_user_from_db(_conn, user.name, user.password)
    .await
    .map_err(internal_error)?;
  Ok(Json(serde_json::json!({
    "id": user.id,
    "name": user.name,
    "createdAt": user.created_at,
    "authToken": create_jwt(user.id).unwrap()
  })))
}

pub async fn get_user(
  State(pool): State<ConnectionPool>,
  Extension(user_id): Extension<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
  let _conn = pool.get().await.map_err(internal_error)?;
  let user = get_user_by_id(_conn, user_id)
    .await
    .map_err(internal_error)?;
  Ok(Json(serde_json::json!({
    "id": user.id,
    "name": user.name,
    "createdAt": user.created_at,
  })))
}


