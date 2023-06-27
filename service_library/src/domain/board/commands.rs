use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::Mutex;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    adapters::repositories::TRepository,
    domain::{
        builder::{Buildable, Builder},
        commands::Command,
    },
    services::{handlers::Future, unit_of_work::UnitOfWork},
};

use super::{entity::BoardState, BoardAggregate};

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct CreateBoard {
    pub author: Uuid,
    pub title: String,
    pub content: String,
    pub state: BoardState,
}

impl Command for CreateBoard {
    type Response = String;
    fn handle(self, uow: Arc<Mutex<UnitOfWork>>) -> Future<Self::Response> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;
            let builder = BoardAggregate::builder();
            let mut board_aggregate: BoardAggregate = builder.build();
            board_aggregate.create_board(self);
            let res = uow.boards.add(board_aggregate).await?;
            uow.commit().await?;
            Ok(res)
        })
    }
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct EditBoard {
    pub id: Uuid,
    pub title: Option<String>,
    pub content: Option<String>,
    pub state: Option<BoardState>,
}
impl Command for EditBoard {
    type Response = ();
    fn handle(self, uow: Arc<Mutex<UnitOfWork>>) -> Future<Self::Response> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;
            let mut board_aggregate = uow.boards.get(&self.id.to_string()).await?;
            board_aggregate.update_board(self);
            uow.boards.update(board_aggregate).await?;
            uow.commit().await?;
            Ok(())
        })
    }
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct AddComment {
    pub board_id: Uuid,
    pub author: Uuid,
    pub content: String,
}

impl Command for AddComment {
    type Response = ();
    fn handle(self, uow: Arc<Mutex<UnitOfWork>>) -> Future<Self::Response> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;
            let mut board_aggregate = uow.boards.get(&self.board_id.to_string()).await?;
            board_aggregate.add_comment(self);
            uow.boards.update(board_aggregate).await?;
            uow.commit().await?;
            Ok(())
        })
    }
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct EditComment {
    pub board_id: Uuid,
    pub id: Uuid,
    pub content: String,
}

impl Command for EditComment {
    type Response = ();
    fn handle(self, uow: Arc<Mutex<UnitOfWork>>) -> Future<Self::Response> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;
            let mut board_aggregate = uow.boards.get(&self.board_id.to_string()).await?;
            board_aggregate.edit_comment(self)?;
            uow.boards.update(board_aggregate).await?;
            uow.commit().await?;
            Ok(())
        })
    }
}
