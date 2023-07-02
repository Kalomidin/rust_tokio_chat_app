use axum::{routing::get, routing::post, Router};

use axum::middleware;
use dotenv::dotenv;
use rust_tokio_chat_app::auth::guard;
use rust_tokio_chat_app::db::setup_conn_pool;
use rust_tokio_chat_app::routes::room::{create_room, join_room};
use rust_tokio_chat_app::routes::user::{get_user, login, signup};

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
  dotenv().ok();

  // set up connection pool
  let pool = setup_conn_pool().await;

  // build our application with some routes
  let app = Router::new()
    .route("/users", get(get_user))
    .route("/rooms/create", post(create_room))
    .route("/rooms/join/:room_id", post(join_room))
    .route_layer(middleware::from_fn(guard))
    .route("/users/signup", post(signup))
    .route("/users/login", post(login))
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
