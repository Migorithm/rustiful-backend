use std::pin::Pin;

use crate::adapters::outbox::Outbox;
use crate::adapters::repositories::TRepository;
use crate::domain::auth::events::AuthEvent;
use crate::domain::board::commands::{AddComment, CreateBoard, EditBoard, EditComment};
use crate::domain::board::events::BoardEvent;
use crate::domain::board::BoardAggregate;

use crate::domain::builder::{Buildable, Builder};
use crate::domain::commands::ServiceResponse;
use crate::utils::ApplicationResult;

use super::unit_of_work::AtomicUnitOfWork;

pub type Future<T> = Pin<Box<dyn futures::Future<Output = ApplicationResult<T>> + Send>>;

pub type CommandHandler<Command, Response> =
    Box<dyn Fn(Command, AtomicUnitOfWork) -> Future<Response> + Send>;
pub type EventHandler<T> = fn(T, AtomicUnitOfWork) -> Future<()>;

pub struct ServiceHandler;
impl ServiceHandler {
    pub fn create_board(cmd: CreateBoard, uow: AtomicUnitOfWork) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;
            let builder = BoardAggregate::builder();
            let mut board_aggregate: BoardAggregate = builder.build();
            board_aggregate.create_board(cmd);
            let res = uow.boards.add(board_aggregate).await?;
            uow.commit().await?;
            Ok(res.into())
        })
    }

    pub fn edit_board(cmd: EditBoard, uow: AtomicUnitOfWork) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;
            let mut board_aggregate = uow.boards.get(&cmd.id.to_string()).await?;
            board_aggregate.update_board(cmd);
            uow.boards.update(board_aggregate).await?;
            uow.commit().await?;
            Ok(().into())
        })
    }

    pub fn add_comment(cmd: AddComment, uow: AtomicUnitOfWork) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;
            let mut board_aggregate = uow.boards.get(&cmd.board_id.to_string()).await?;
            board_aggregate.add_comment(cmd);
            uow.boards.update(board_aggregate).await?;
            uow.commit().await?;
            Ok(().into())
        })
    }

    pub fn edit_comment(cmd: EditComment, uow: AtomicUnitOfWork) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;
            let mut board_aggregate = uow.boards.get(&cmd.board_id.to_string()).await?;
            board_aggregate.edit_comment(cmd)?;
            uow.boards.update(board_aggregate).await?;
            uow.commit().await?;
            Ok(().into())
        })
    }

    pub fn handle_outbox(outbox: Outbox, uow: AtomicUnitOfWork) -> Future<ServiceResponse> {
        Box::pin(async move {
            let msg = outbox.convert_event();
            let mut uow = uow.lock().await;
            uow.begin().await;

            // ! Todo msg handling logic
            outbox.update(&mut uow.connection).await?;

            uow.commit().await?;
            Ok(true.into())
        })
    }
}

pub(crate) static BOARD_CREATED_EVENT_HANDLERS: [EventHandler<BoardEvent>; 0] = [];
pub(crate) static BOARD_UPDATED_EVENT_HANDLERS: [EventHandler<BoardEvent>; 0] = [];
pub(crate) static COMMENT_ADDED_EVENT_HANDLERS: [EventHandler<BoardEvent>; 0] = [];

pub(crate) static ACCOUNT_CREATED_EVENT_HANDLERS: [EventHandler<AuthEvent>; 0] = [];
pub(crate) static ACCOUNT_UPDATED_EVENT_HANDLERS: [EventHandler<AuthEvent>; 0] = [];
