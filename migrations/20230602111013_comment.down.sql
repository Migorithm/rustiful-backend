-- Add down migration script here
DROP TABLE IF EXISTS community_comment;

DROP TYPE IF EXISTS comment_state;