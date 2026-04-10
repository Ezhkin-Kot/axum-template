CREATE TABLE IF NOT EXISTS "users" (
  "id" UUID NOT NULL PRIMARY KEY,
  "name" varchar NOT NULL,
  "role" text NOT NULL,
  "email" varchar NOT NULL UNIQUE,
  "password_hash" varchar NOT NULL
);

CREATE TABLE IF NOT EXISTS "refresh_tokens"(
  "user_id" INT NOT NULL,
  "token" text NOT NULL
);

