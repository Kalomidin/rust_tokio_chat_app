use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

#[derive(Clone, Serialize, Deserialize)]
pub struct Member {
  pub id: i64,
  pub room_id: i64,
  pub user_id: i64,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub last_joined_at: chrono::DateTime<chrono::Utc>,
  pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn create_new_member(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  room_id: i64,
  user_id: i64,
) -> Result<Member, tokio_postgres::Error> {
  let query = "INSERT INTO room_member (room_id, user_id) VALUES ($1, $2)";
  conn.execute(query, &[&room_id, &user_id]).await?;
  let room = get_member(conn, room_id, user_id).await?;
  Ok(room)
}

pub async fn get_member(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  room_id: i64,
  user_id: i64,
) -> Result<Member, tokio_postgres::Error> {
  let query =
    "SELECT * FROM room_member WHERE room_id = $1 AND user_id = $2  AND deleted_at is NULL";
  let row = conn.query_one(query, &[&room_id, &user_id]).await?;
  Ok(row_to_member(row))
}

pub async fn get_member_by_id(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  id: i64,
) -> Result<Member, tokio_postgres::Error> {
  let query = "SELECT * FROM room_member WHERE id = $1  AND deleted_at is NULL";
  let row = conn.query_one(query, &[&id]).await?;
  Ok(row_to_member(row))
}

pub async fn delete_member(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  room_id: i64,
  user_id: i64,
) -> Result<Member, tokio_postgres::Error> {
  let query = "UPDATE room_member SET deleted_at = NOW() WHERE room_id = $1 AND user_id = $2 AND deleted_at is NULL";
  conn.execute(query, &[&room_id, &user_id]).await?;
  let room = get_member(conn, room_id, user_id).await?;
  Ok(room)
}

pub async fn count_active_members(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  room_id: i64,
) -> Result<i64, tokio_postgres::Error> {
  let query = "SELECT COUNT(*) FROM room_member WHERE room_id = $1 AND deleted_at is NULL";
  let rows = conn.query(query, &[&room_id]).await?;
  let count: i64 = rows[0].get(0);
  Ok(count)
}

pub async fn update_last_joined_at(
  conn: &mut PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  room_id: i64,
  user_id: i64,
) -> Result<Member, tokio_postgres::Error> {
  let query = "UPDATE room_member SET last_joined_at = NOW() WHERE room_id = $1 AND user_id = $2 AND deleted_at is NULL";
  conn.execute(query, &[&room_id, &user_id]).await?;
  let room = get_member(conn, room_id, user_id).await?;
  Ok(room)
}

fn row_to_member(row: tokio_postgres::Row) -> Member {
  let id: i64 = row.get(0);
  let room_id: i64 = row.get(1);
  let user_id: i64 = row.get(2);
  let created_at: DateTime<chrono::Utc> = row.get(3);
  let last_joined_at: DateTime<chrono::Utc> = row.get(4);
  let deleted_at: Option<DateTime<chrono::Utc>> = row.get(5);
  Member {
    id,
    room_id,
    user_id,
    created_at,
    last_joined_at,
    deleted_at,
  }
}
