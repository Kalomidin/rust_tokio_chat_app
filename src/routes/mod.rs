use axum::{
  http::{StatusCode},
};
use bb8::{Pool};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub mod models;
pub mod room;
pub mod user;

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
  E: std::error::Error,
{
  (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
