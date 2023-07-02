use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

#[derive(Clone, Serialize, Deserialize)]
pub struct Room {
  pub id: i64,
  pub name: String,
  pub created_by: i64,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn create_new_room(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  name: String,
  created_by: i64,
) -> Result<Room, tokio_postgres::Error> {
  let query = "INSERT INTO room (name, created_by) VALUES ($1, $2)";
  conn.execute(query, &[&name, &created_by]).await?;
  let room = get_room(conn, name, created_by).await?;
  Ok(room)
}

pub async fn get_room(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  name: String,
  created_by: i64,
) -> Result<Room, tokio_postgres::Error> {
  let query = "SELECT * FROM room WHERE name = $1 AND created_by = $2";
  let row = conn.query_one(query, &[&name, &created_by]).await?;
  Ok(row_to_room(row))
}

pub async fn get_room_by_id(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  id: i64,
) -> Result<Room, tokio_postgres::Error> {
  let query = "SELECT * FROM room WHERE id = $1";
  let row = conn.query_one(query, &[&id]).await?;
  Ok(row_to_room(row))
}

fn row_to_room(row: tokio_postgres::Row) -> Room {
  let id: i64 = row.get(0);
  let name: String = row.get(1);
  let created_by: i64 = row.get(2);
  let created_at: DateTime<chrono::Utc> = row.get(3);
  let deleted_at: Option<DateTime<chrono::Utc>> = row.get(4);
  Room {
    id,
    name,
    created_by,
    created_at,
    deleted_at,
  }
}
