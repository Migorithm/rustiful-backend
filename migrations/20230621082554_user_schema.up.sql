-- Add up migration script here

CREATE TYPE account_state AS ENUM (
    'VerificationRequired', 'Created', 'Deleted', 'Blocked'
);

CREATE TABLE IF NOT EXISTS auth_account(
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    state account_state NOT NULL,
    hashed_password TEXT NOT NULL,
    nickname TEXT NOT NULL,
    create_dt TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL DEFAULT 0
);



CREATE TABLE IF NOT EXISTS auth_token_stat(
    account_id TEXT PRIMARY KEY,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL,
    create_dt TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 min'),
    CONSTRAINT fk_account_id
        FOREIGN KEY(account_id)
        REFERENCES auth_account(id)
        ON DELETE CASCADE
);
