use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug)]
pub struct ServiceError {
  code: StatusCode,
  message: String,
}

impl ServiceError {
  pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
    Self {
      code,
      message: message.into(),
    }
  }

  pub fn message(&self) -> &str {
    &self.message
  }
}

impl IntoResponse for ServiceError {
  fn into_response(self) -> axum::response::Response {
    (
      self.code,
      Json(ResponseMessage {
        message: self.message,
      }),
    )
      .into_response()
  }
}

#[derive(Serialize)]
struct ResponseMessage {
  message: String,
}

pub fn internal_error_to_service_error<E>(err: E) -> ServiceError
where
  E: std::error::Error,
{
  ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

pub fn db_error_to_service_error(err: tokio_postgres::Error) -> ServiceError {
  let _row_count_err = "query returned an unexpected number of rows";
  match err.code() {
    Some(sql_state) => match sql_state.code() {
      "23503" => ServiceError::new(StatusCode::BAD_REQUEST, "foreign key violation"),
      "23505" => ServiceError::new(StatusCode::BAD_REQUEST, "unique key violation"),
      "P0002" => ServiceError::new(StatusCode::BAD_REQUEST, "Invalid input"),
      _ => ServiceError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Database error: ".to_owned() + sql_state.code(),
      ),
    },
    _ => match err.to_string().as_str() {
      _row_count_err => ServiceError::new(
        StatusCode::BAD_REQUEST,
        "Bad Request: err: row_count_err, most likely data is not found",
      ),
      _ => ServiceError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Database error: ".to_owned() + &err.to_string(),
      ),
    },
  }
}
