use axum::{routing::delete, routing::get, routing::post, Extension, Router};

use axum::middleware;
use dotenv::dotenv;

use rust_tokio_chat_app::auth::guard;
use rust_tokio_chat_app::db::setup_conn_pool;
use rust_tokio_chat_app::routes::room::{create_room, join_room, leave_room, remove_member};
use rust_tokio_chat_app::routes::user::{get_user, login, signup};
use rust_tokio_chat_app::ws::lobby::Lobby;
use std::sync::Arc;

//allows to extract the IP of connecting user

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
  dotenv().ok();

  // set up connection pool
  let pool = setup_conn_pool().await;

  let app_state = Arc::new(Lobby::new(pool.clone()));

  // build our application with some routes
  let app = Router::new()
    .route("/users", get(get_user))
    .route("/rooms/create", post(create_room))
    .route("/rooms/leave/:room_id", post(leave_room))
    .route("/rooms/remove/:room_id", delete(remove_member))
    .route("/rooms/join/:room_id", get(join_room))
    .route_layer(middleware::from_fn_with_state(pool.clone(), guard))
    .route("/users/signup", post(signup))
    .route("/users/login", post(login))
    .route("/health", get(heath_check))
    .layer(Extension(app_state))
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
