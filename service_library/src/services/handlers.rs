use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::adapters::database::ContextManager;
use crate::adapters::outbox::Outbox;
use crate::adapters::repositories::{Repository, TRepository};

use crate::domain::board::commands::{AddComment, CreateBoard, EditBoard, EditComment};

use crate::domain::board::BoardAggregate;

use crate::domain::board::events::BoardCreated;
use crate::domain::builder::{Buildable, Builder};
use crate::domain::commands::ServiceResponse;
use crate::utils::ApplicationResult;

use super::unit_of_work::UnitOfWork;
pub type Future<T> = Pin<Box<dyn futures::Future<Output = ApplicationResult<T>> + Send>>;

pub struct ServiceHandler;
impl ServiceHandler {
    pub fn create_board(
        cmd: CreateBoard,
        context: Arc<RwLock<ContextManager>>,
    ) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                context.read().await.pool,
            );
            uow.begin().await.unwrap();
            let builder = BoardAggregate::builder();
            let mut board_aggregate: BoardAggregate = builder.build();
            board_aggregate.create_board(cmd);
            let res = uow.repository.add(&mut board_aggregate).await?;
            uow.commit().await?;
            Ok(res.into())
        })
    }

    pub fn edit_board(
        cmd: EditBoard,
        context: Arc<RwLock<ContextManager>>,
    ) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                context.read().await.pool,
            );
            uow.begin().await.unwrap();
            let mut board_aggregate = uow.repository.get(&cmd.id.to_string()).await?;
            board_aggregate.update_board(cmd);
            uow.repository.update(&mut board_aggregate).await?;
            uow.commit().await?;

            Ok(().into())
        })
    }

    pub fn add_comment(
        cmd: AddComment,
        context: Arc<RwLock<ContextManager>>,
    ) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                context.read().await.pool,
            );
            uow.begin().await.unwrap();
            let mut board_aggregate = uow.repository.get(&cmd.board_id.to_string()).await?;
            board_aggregate.add_comment(cmd);
            uow.repository.update(&mut board_aggregate).await?;
            uow.commit().await?;
            Ok(().into())
        })
    }

    pub fn edit_comment(
        cmd: EditComment,
        context: Arc<RwLock<ContextManager>>,
    ) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                context.read().await.pool,
            );
            uow.begin().await.unwrap();
            let mut board_aggregate = uow.repository.get(&cmd.board_id.to_string()).await?;
            board_aggregate.edit_comment(cmd)?;
            uow.repository.update(&mut board_aggregate).await?;
            uow.commit().await?;
            Ok(().into())
        })
    }

    pub fn handle_outbox(
        outbox: Outbox,
        context: Arc<RwLock<ContextManager>>,
    ) -> Future<ServiceResponse> {
        Box::pin(async move {
            let _msg = outbox.convert_event();

            let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                context.read().await.pool,
            );

            uow.begin().await.unwrap();

            // ! Todo msg handling logic
            outbox.update(uow.executor.clone()).await?;

            uow.commit().await?;
            Ok(true.into())
        })
    }
}

pub struct EventHandler;
impl EventHandler {
    pub fn test_event_handler(
        _event: BoardCreated,
        context: Arc<RwLock<ContextManager>>,
        some_dependency: fn(String, i32) -> ServiceResponse,
    ) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                context.read().await.pool,
            );
            uow.begin().await.unwrap();
            println!("You got here!");
            uow.commit().await?;

            Ok(some_dependency("well..".into(), 1))
        })
    }
    pub fn test_event_handler2(
        _event: BoardCreated,
        context: Arc<RwLock<ContextManager>>,
    ) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                context.read().await.pool,
            );
            uow.begin().await.unwrap();
            println!("You got here too!");
            uow.commit().await?;
            Ok(ServiceResponse::Empty(()))
        })
    }
}
