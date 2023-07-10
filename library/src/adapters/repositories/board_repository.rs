
use crate::domain::board::entity::{Board, BoardState, Comment, CommentState};

use crate::domain::board::BoardAggregate;

use crate::domain::{builder::*};

use crate::utils::ApplicationError;

use std::mem;

use std::str::FromStr;

use uuid::Uuid;

use super::{Repository, TRepository};


impl Repository<BoardAggregate> {
    pub async fn add(&mut self, aggregate: &mut BoardAggregate) -> Result<String, ApplicationError> {
        self.set_events(mem::take(&mut aggregate.events));
        self._add(aggregate).await
    }
    pub async fn update(&mut self, aggregate: &mut BoardAggregate) -> Result<(), ApplicationError> {
        self.set_events(mem::take(&mut aggregate.events));
        self._update(aggregate).await
    }

    pub async fn _add(&mut self, aggregate: &BoardAggregate) -> Result<String, ApplicationError> {
        let board = &aggregate.board;

        sqlx::query_as!(
            Board,
            "INSERT INTO community_board (id, author, title, content, tags,state) VALUES ($1, $2, $3, $4, $5,$6)",
            board.id,
            board.author,
            &board.title,
            &board.content,
            &board.tags,
            board.state.clone() as BoardState,
        ).execute(self.executor.write().await.transaction()).await.map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;

        Ok(board.id.to_string())
    }

    pub async fn get(&self, _aggregate_id: &str) -> Result<BoardAggregate, ApplicationError> {
        let uuidfied = Uuid::from_str(_aggregate_id).unwrap();

        let board = sqlx::query_as!(
            Board,
            r#"
            SELECT 
                id,
                author,
                title,
                content,
                state AS "state: BoardState",
                create_dt,
                tags,
                version
            FROM community_board
            WHERE id = $1
            "#,
            uuidfied
        )
        .fetch_one(self.executor.read().await.connection())
        .await
        .map_err(|err| {
            eprintln!("{}", err);
            ApplicationError::DatabaseConnectionError(Box::new(err))
        })?;

        let comments = sqlx::query_as!(
            Comment,
            r#"
            SELECT 
                id,
                board_id,
                author,
                content,
                state as "state:CommentState",
                create_dt
            FROM community_comment
            WHERE board_id = $1
            "#,
            uuidfied,
        )
        .fetch_all(self.executor.read().await.connection())
        .await
        .map_err(|err| {
            eprintln!("{}", err);
            ApplicationError::DatabaseConnectionError(Box::new(err))
        })?;

        //*  Build board aggregate
        let board_aggregate_builder = BoardAggregate::builder();
        let board_aggregate = board_aggregate_builder
            .take_board(board)
            .take_comments(comments)
            .build();
        Ok(board_aggregate)
    }

    pub async fn _update(&mut self, aggregate: &BoardAggregate) -> Result<(), ApplicationError> {
        let board = &aggregate.board;

        let mut to_be_added_comment: Option<&Comment> = None;
        let mut to_be_updated_comment: Option<&Comment> = None;

        for comment in aggregate.comments.iter() {
            if comment.state == CommentState::Pending {
                to_be_added_comment = Some(comment);
                break;
            }
            if comment.state == CommentState::UpdatePending {
                to_be_updated_comment = Some(comment);
                break;
            }
        }

        // * Update Board
        sqlx::query_as!(
            Board,
            "UPDATE community_board SET 
            author = $1,
            title = $2,
            content = $3,
            state = $4,
            version = $5
            WHERE id = $6 AND version = $7",
            board.author,
            board.title,
            board.content,
            board.state.clone() as BoardState,
            board.version + 1,
            board.id,
            board.version
        )
        .execute(self.executor.write().await.transaction())
        .await
        .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;

        // * Insert Comment
        if let Some(comment) = to_be_added_comment {
            sqlx::query_as!(
                Comment,
                "INSERT INTO community_comment (
                    id,
                    board_id,
                    author,
                    content,
                    state,
                    create_dt
                ) 
                VALUES ($1, $2, $3, $4, $5, $6)",
                comment.id,
                comment.board_id,
                comment.author,
                comment.content,
                CommentState::Created as CommentState,
                comment.create_dt
            )
            .execute(self.executor.write().await.transaction())
            .await
            .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;
        }

        if let Some(comment) = to_be_updated_comment {
            sqlx::query_as!(
                Comment,
                r#"UPDATE community_comment SET 
                    content =$1
                WHERE id = $2"#,
                comment.content,
                comment.id
            )
            .execute(self.executor.write().await.transaction())
            .await
            .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;
        }

        Ok(())
    }
}