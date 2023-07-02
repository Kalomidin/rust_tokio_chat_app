-- Your SQL goes here
CREATE TABLE users (
  id bigserial NOT NULL PRIMARY KEY,
  name VARCHAR NOT NULL,
  password VARCHAR NOT NULL,
  created_at timestamp with time zone DEFAULT now() NOT NULL,
  unique(name)
)
