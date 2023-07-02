use crate::helpers::get_env;

use bb8::{ManageConnection, Pool};
use bb8_postgres::PostgresConnectionManager;

use tokio_postgres::{config::Config, NoTls};
use tokio_postgres_migration::Migration;

pub mod user;

const SCRIPTS_UP: [(&str, &str); 4] = [
  (
    "users",
    include_str!("../../migrations/2023-07-02-011854_user/up.sql"),
  ),
  (
    "room",
    include_str!("../../migrations/2023-07-02-012038_room/up.sql"),
  ),
  (
    "member",
    include_str!("../../migrations/2023-07-02-012049_member/up.sql"),
  ),
  (
    "message",
    include_str!("../../migrations/2023-07-02-012056_message/up.sql"),
  ),
];

const SCRIPTS_DOWN: [(&str, &str); 4] = [
  (
    "users",
    include_str!("../../migrations/2023-07-02-011854_user/down.sql"),
  ),
  (
    "room",
    include_str!("../../migrations/2023-07-02-012038_room/down.sql"),
  ),
  (
    "member",
    include_str!("../../migrations/2023-07-02-012049_member/down.sql"),
  ),
  (
    "message",
    include_str!("../../migrations/2023-07-02-012056_message/down.sql"),
  ),
];

pub async fn setup_conn_pool() -> Pool<PostgresConnectionManager<NoTls>> {
  let postgres_user = get_env("POSTGRES_USER");
  let postgres_password = get_env("POSTGRES_PASSWORD");
  let postgres_host = get_env("POSTGRES_HOST");
  let postgres_port = get_env("POSTGRES_PORT");
  let postgres_db = get_env("POSTGRES_DB");
  let _params = format!(
    "user={} password={} host={} port={} dbname={}",
    postgres_user, postgres_password, postgres_host, postgres_port, postgres_db
  );
  let mut config = Config::default();
  config
    .host(&postgres_host)
    .port(postgres_port.parse::<u16>().unwrap())
    .user(&postgres_user)
    .password(postgres_password)
    .dbname(&postgres_db);
  let manager = PostgresConnectionManager::new(config, NoTls);
  let mut connection = manager.connect().await.unwrap();
  match run_migrations(&mut connection).await {
    Ok(_) => (),
    Err(e) => {
      println!("Error running migrations: {}", e);
      std::process::exit(1);
    }
  };

  let pool = Pool::builder().build(manager).await.unwrap();
  return pool;
}

pub async fn run_migrations(
  client: &mut tokio_postgres::Client,
) -> Result<(), tokio_postgres::Error> {
  let migration = Migration::new("schema_migrations".to_string());
  // execute non existing migrations
  migration.up(client, &SCRIPTS_UP).await?;
  Ok(())
}
