use super::models::{LoginRequest, NewUserRequest};
use crate::auth::create_jwt;
use crate::db::user::{get_user as get_user_from_db, get_user_by_id, insert_new_user};
use crate::errors::{db_error_to_service_error, internal_error_to_service_error, ServiceError};
use crate::ConnectionPool;
use axum::{extract::Extension, extract::State, Json};

pub async fn signup(
  State(pool): State<ConnectionPool>,
  Json(user): Json<NewUserRequest>,
) -> Result<Json<serde_json::Value>, ServiceError> {
  let mut conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let user = insert_new_user(&mut conn, user.name, user.password)
    .await
    .map_err(db_error_to_service_error)?;
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
) -> Result<Json<serde_json::Value>, ServiceError> {
  let mut conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let user = get_user_from_db(&mut conn, user.name, user.password)
    .await
    .map_err(db_error_to_service_error)?;
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
) -> Result<Json<serde_json::Value>, ServiceError> {
  let mut conn = pool.get().await.map_err(internal_error_to_service_error)?;
  let user = get_user_by_id(&mut conn, user_id)
    .await
    .map_err(db_error_to_service_error)?;
  Ok(Json(serde_json::json!({
    "id": user.id,
    "name": user.name,
    "createdAt": user.created_at,
  })))
}
