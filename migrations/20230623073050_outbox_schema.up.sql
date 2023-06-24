-- Add up migration script here



CREATE TABLE IF NOT EXISTS service_outbox(
    id UUID PRIMARY KEY,
    aggregate_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    state TEXT NOT NULL,
    processed boolean NOT NULL,
    create_dt TIMESTAMPTZ NOT NULL DEFAULT NOW()
);



