use super::models::NewUserRequest;
use super::{internal_error, ConnectionPool};
use axum::{
  extract::{State},
  http::{StatusCode},
  Json,
};





async fn create_user(
  State(pool): State<ConnectionPool>,
  Json(_user): Json<NewUserRequest>,
) -> Result<String, (StatusCode, String)> {
  let _conn = pool.get().await.map_err(internal_error)?;

  Ok("OK".to_string())
}
