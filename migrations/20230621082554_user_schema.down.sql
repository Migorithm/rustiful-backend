-- Add down migration script here

-- Add down migration script here


DROP TABLE IF EXISTS auth_account CASCADE;

DROP TABLE IF EXISTS auth_token_stat;

DROP TYPE IF EXISTS account_state;