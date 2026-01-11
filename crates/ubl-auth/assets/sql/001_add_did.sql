-- assets/sql/001_add_did.sql (SQLite/D1)
ALTER TABLE users ADD COLUMN did TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS users_did_uq ON users(did) WHERE did IS NOT NULL;

CREATE TABLE IF NOT EXISTS identity_links (
  did TEXT NOT NULL,
  provider TEXT NOT NULL,
  provider_user_id TEXT NOT NULL,
  PRIMARY KEY (provider, provider_user_id)
);
