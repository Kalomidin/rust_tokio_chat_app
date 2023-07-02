use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
  pub id: i64,
  pub room_id: i64,
  pub sender_id: i64,
  pub msg: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn add_message(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  room_id: i64,
  sender_id: i64,
  msg: String,
) -> Result<(), tokio_postgres::Error> {
  let query = "INSERT INTO messages (room_id, sender_id, msg) VALUES ($1, $2, $3)";
  conn
    .execute(query, &[&room_id, &sender_id, &msg])
    .await?;
  Ok(())
}

pub async fn get_unread_messages(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  room_id: i64,
  last_seen_at: DateTime<chrono::Utc>,
) -> Result<Vec<Message>, tokio_postgres::Error> {
  let query =
    "SELECT * FROM messages WHERE room_id = $1 AND created_at > $2 ORDER BY created_at ASC";
  let rows = conn.query(query, &[&room_id, &last_seen_at]).await?;
  let mut messages = Vec::new();
  for row in rows {
    messages.push(row_to_message(row));
  }
  Ok(messages)
}

fn row_to_message(row: tokio_postgres::Row) -> Message {
  Message {
    id: row.get(0),
    room_id: row.get(1),
    sender_id: row.get(2),
    msg: row.get(4),
    created_at: row.get(5),
  }
}
