use axum::{
  routing::get,
  Router,
};


use dotenv::dotenv;
use rust_tokio_chat_app::db::setup_conn_pool;

use std::net::SocketAddr;


#[tokio::main]
async fn main() {
  dotenv().ok();

  // set up connection pool
  let pool = setup_conn_pool().await;

  // build our application with some routes
  let app = Router::new()
    .route("/health", get(heath_check))
    .with_state(pool);

  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}

async fn heath_check() -> &'static str {
  "OK"
}
