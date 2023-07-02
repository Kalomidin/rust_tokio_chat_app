-- Your SQL goes here
CREATE TABLE room_member (
  id bigserial NOT NULL PRIMARY KEY,
  room_id bigint NOT NULL  REFERENCES room(id),
  user_id bigint NOT NULL  REFERENCES users(id),
  created_at timestamp with time zone DEFAULT now() NOT NULL,
  last_joined_at timestamp with time zone DEFAULT now() NOT NULL,
  deleted_at timestamp with time zone DEFAULT NULL,
  unique (room_id, user_id)
)