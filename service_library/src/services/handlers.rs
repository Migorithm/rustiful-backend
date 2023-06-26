use std::{pin::Pin, sync::Arc};

use crate::adapters::repositories::TRepository;
use crate::domain::auth::events::AuthEvent;
use crate::domain::board::events::BoardEvent;
use crate::domain::builder::{Buildable, Builder};

use crate::domain::commands::{Command, ServiceResponse};
use crate::{
    domain::{board::BoardAggregate, commands::ApplicationCommand},
    utils::ApplicationResult,
};

use tokio::sync::Mutex;

use super::unit_of_work::UnitOfWork;

pub type Future<T> = Pin<Box<dyn futures::Future<Output = ApplicationResult<T>> + Send>>;

pub trait Handler {
    type Command: Command<Response = Self::Response, Handler = Self>;
    type Response;

    fn execute(cmd: Self::Command, uow: Arc<Mutex<UnitOfWork>>) -> Future<Self::Response>;
}

pub struct ServiceHandler;

impl Handler for ServiceHandler {
    type Command = ApplicationCommand;
    type Response = ServiceResponse;
    fn execute(cmd: Self::Command, uow: Arc<Mutex<UnitOfWork>>) -> Future<Self::Response> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;

            let res = match cmd {
                Self::Command::CreateBoard { .. } => {
                    let builder = BoardAggregate::builder();
                    let mut board_aggregate: BoardAggregate = builder.build();
                    board_aggregate.execute(cmd)?;
                    uow.boards.add(board_aggregate).await?
                }
                Self::Command::EditBoard { id, .. } => {
                    let mut board_aggregate = uow.boards.get(&id.to_string()).await?;
                    board_aggregate.execute(cmd)?;
                    uow.boards.update(board_aggregate).await?;
                    id.to_string()
                }
                Self::Command::AddComment { board_id, .. } => {
                    let mut board_aggregate = uow.boards.get(&board_id.to_string()).await?;
                    board_aggregate.execute(cmd)?;
                    uow.boards.update(board_aggregate).await?;
                    board_id.to_string()
                }
                Self::Command::EditComment { board_id, .. } => {
                    let mut board_aggregate = uow.boards.get(&board_id.to_string()).await?;
                    board_aggregate.execute(cmd)?;
                    uow.boards.update(board_aggregate).await?;
                    board_id.to_string()
                }
            };
            uow.commit().await?;
            Ok(res.into())
        })
    }
}

pub type CommandHandler<Command, Response> =
    Box<dyn Fn(Command, Arc<Mutex<UnitOfWork>>) -> Future<Response> + Send>;
pub type EventHandler<T> = fn(T, Arc<Mutex<UnitOfWork>>) -> Future<()>;

pub(crate) static BOARD_CREATED_EVENT_HANDLERS: [EventHandler<BoardEvent>; 0] = [];
pub(crate) static BOARD_UPDATED_EVENT_HANDLERS: [EventHandler<BoardEvent>; 0] = [];
pub(crate) static COMMENT_ADDED_EVENT_HANDLERS: [EventHandler<BoardEvent>; 0] = [];

pub(crate) static ACCOUNT_CREATED_EVENT_HANDLERS: [EventHandler<AuthEvent>; 0] = [];
pub(crate) static ACCOUNT_UPDATED_EVENT_HANDLERS: [EventHandler<AuthEvent>; 0] = [];
