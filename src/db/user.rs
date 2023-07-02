use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use chrono::{DateTime, TimeZone};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
  pub id: i64,
  pub name: String,
  pub password: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn insert_new_user(
  conn: PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  name: String,
  password: String,
) -> Result<User, tokio_postgres::Error> {
  let query = "INSERT INTO users (name, password) VALUES ($1, $2)";
  conn.execute(query, &[&name, &password]).await?;
  let user = get_user(conn, name, password).await?;
  Ok(user)
}

pub async fn get_user(
  conn: PooledConnection<'_, PostgresConnectionManager<NoTls>>,
  name: String,
  password: String,
) -> Result<User, tokio_postgres::Error> {
  let query = "SELECT * FROM users WHERE name = $1 AND password = $2";
  let row = conn.query_one(query, &[&name, &password]).await?;
  Ok(row_to_user(row))
}

pub async fn get_user_by_id(
    conn: PooledConnection<'_, PostgresConnectionManager<NoTls>>,
    id: i64,
  ) -> Result<User, tokio_postgres::Error> {
    let query = "SELECT * FROM users WHERE id = $1";
    let row = conn.query_one(query, &[&id]).await?;
    Ok(row_to_user(row))
  }


fn row_to_user(row: tokio_postgres::Row) -> User {
    let id: i64 = row.get(0);
    let name: String = row.get(1);
    let password: String = row.get(2);
    let created_at: DateTime<chrono::Utc> = row.get(3);
    User {
      id,
      name,
      password,
      created_at,
    }
  }