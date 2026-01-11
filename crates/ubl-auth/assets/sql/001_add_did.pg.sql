-- assets/sql/001_add_did.pg.sql (PostgreSQL)
ALTER TABLE users ADD COLUMN IF NOT EXISTS did text;
CREATE UNIQUE INDEX IF NOT EXISTS users_did_uq ON users(did);

CREATE TABLE IF NOT EXISTS identity_links (
  did text NOT NULL,
  provider text NOT NULL,
  provider_user_id text NOT NULL,
  PRIMARY KEY (provider, provider_user_id)
);
