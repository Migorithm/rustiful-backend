use crate::adapters::database::AtomicConnection;

use crate::domain::board::entity::{Board, BoardState, Comment, CommentState};

use crate::domain::board::BoardAggregate;
use crate::domain::{builder::*, Message};

use crate::utils::ApplicationError;
use async_trait::async_trait;
use std::borrow::Borrow;

use std::collections::VecDeque;
use std::str::FromStr;

use uuid::Uuid;

use super::{Repository, TRepository};

#[async_trait]
impl TRepository for Repository<BoardAggregate> {
    type Aggregate = BoardAggregate;

    fn new(connection: AtomicConnection) -> Self {
        Self {
            connection,
            _phantom: Default::default(),
            events: Default::default(),
        }
    }
    fn get_events(&self) -> &VecDeque<Box<dyn Message>> {
        &self.events
    }
    fn set_events(&mut self, events: VecDeque<Box<dyn Message>>) {
        self.events = events
    }

    fn connection(&self) -> &AtomicConnection {
        &self.connection
    }

    async fn _add(
        &mut self,
        aggregate: impl AsRef<BoardAggregate> + Send + Sync,
    ) -> Result<String, ApplicationError> {
        let board = &aggregate.as_ref().board;

        sqlx::query_as!(
            Board,
            "INSERT INTO community_board (id, author, title, content, state) VALUES ($1, $2, $3, $4, $5) ",
            board.id,
            board.author,
            &board.title,
            &board.content,
            board.state.clone() as BoardState,
        ).execute(self.connection.write().await.connection()).await.map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;

        Ok(board.id.to_string())
    }

    async fn get(&mut self, _aggregate_id: &str) -> Result<BoardAggregate, ApplicationError> {
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
                version
            FROM community_board
            WHERE id = $1
            "#,
            uuidfied
        )
        .fetch_one(&self.connection.read().await.pool)
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
        .fetch_all(&self.connection.read().await.pool)
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

    async fn _update(
        &mut self,
        aggregate: impl AsRef<BoardAggregate> + Send + Sync,
    ) -> Result<(), ApplicationError> {
        let board = &aggregate.borrow().as_ref().board;

        let mut to_be_added_comment: Option<&Comment> = None;
        let mut to_be_updated_comment: Option<&Comment> = None;

        for comment in aggregate.as_ref().comments.iter() {
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
        .execute(self.connection.write().await.connection())
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
            .execute(self.connection.write().await.connection())
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
            .execute(self.connection.write().await.connection())
            .await
            .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;
        }

        Ok(())
    }
}
