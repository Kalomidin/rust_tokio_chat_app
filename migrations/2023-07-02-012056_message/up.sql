-- Your SQL goes here
CREATE TABLE message (
  id bigserial NOT NULL PRIMARY KEY,
  room_id bigint NOT NULL  REFERENCES room(id),
  --- sender_id is the member who sent the message
  sender_id bigint NOT NULL  REFERENCES room_member(id),
  --- sender message
  msg text NOT NULL,
  created_at timestamp with time zone DEFAULT now() NOT NULL
)