
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
pub mod auth;
pub mod db;
pub mod errors;
pub mod helpers;
pub mod routes;
pub mod ws;

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;