-- Add up migration script here
CREATE TYPE comment_state AS ENUM (
    'Created', 'Deleted' 
);

CREATE TABLE IF NOT EXISTS community_comment(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id UUID NOT NULL,
    author UUID NOT NULL,
    content TEXT NOT NULL,
    state comment_state NOT NULL,
    create_dt TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_board_id
        FOREIGN KEY(board_id)
        REFERENCES community_board(id)
        ON DELETE CASCADE
);



