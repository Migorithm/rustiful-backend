-- Add up migration script here
CREATE TYPE board_state AS ENUM (
    'Unpublished', 'Published', 'Deleted'
);
CREATE TABLE IF NOT EXISTS community_board(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author UUID NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    state board_state NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    create_dt TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

