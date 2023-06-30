use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::pin::Pin;

use crate::adapters::outbox::Outbox;
use crate::adapters::repositories::TRepository;

use crate::domain::board::commands::{AddComment, CreateBoard, EditBoard, EditComment};

use crate::domain::board::BoardAggregate;

use crate::domain::builder::{Buildable, Builder};
use crate::domain::commands::ServiceResponse;
use crate::utils::ApplicationResult;

use super::unit_of_work::AtomicUnitOfWork;
use crate::domain::Message;
pub type Future<T> = Pin<Box<dyn futures::Future<Output = ApplicationResult<T>> + Send>>;

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
            let _msg = outbox.convert_event();
            let mut uow = uow.lock().await;
            uow.begin().await;

            // ! Todo msg handling logic
            outbox.update(&mut uow.connection).await?;

            uow.commit().await?;
            Ok(true.into())
        })
    }
}

pub struct EventHandler;
impl EventHandler {
    pub fn test_event_handler(
        _event: Box<dyn Message>,
        uow: AtomicUnitOfWork,
    ) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;
            println!("You got here!");
            uow.commit().await?;
            Ok(ServiceResponse::Empty(()))
        })
    }
    pub fn test_event_handler2(
        _event: Box<dyn Message>,
        uow: AtomicUnitOfWork,
    ) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = uow.lock().await;
            uow.begin().await;
            println!("You got here too!");
            uow.commit().await?;
            Ok(ServiceResponse::Empty(()))
        })
    }
}

macro_rules! command_handler {
    (
        [$iden:ident, $injectable:ty] ; {$($command:ty:$handler:expr),*}
    )
        => {
        pub fn init_command_handler() -> HashMap::<TypeId,Box<dyn Fn(Box<dyn Any + Send + Sync>, $injectable ) -> Future<ServiceResponse> + Send + Sync>>{
            let mut map: HashMap::<_,Box<dyn Fn(_, _ ) -> Future<_> + Send + Sync>> = HashMap::new();
            $(
                map.insert(
                    TypeId::of::<$command>(),
                    Box::new(
                        |c:Box<dyn Any+Send+Sync>, $iden: $injectable |->Future<ServiceResponse>{
                            $handler(*c.downcast::<$command>().unwrap(),$iden)
                        }
                    )
                );
            )*
            map
        }
    };
}

macro_rules! event_handler {
    (
        [$iden:ident, $injectable:ty] ; {$($event:ty: [$($handler:expr),* ]),*}
    ) =>{
        pub fn init_event_handler() -> HashMap<String, Vec<Box<dyn Fn(Box<dyn Message>, $injectable) -> Future<ServiceResponse> + Send + Sync>>>{
            let mut map : HashMap<String, Vec<Box<dyn Fn(_, _) -> Future<_> + Send + Sync>>> = HashMap::new();
            $(
                map.insert(
                    stringify!($event).into(),
                    vec![
                        $(
                            Box::new(
                                |e, $iden: $injectable| -> Future<ServiceResponse>{
                                    $handler(e,$iden)
                                }
                                ),
                        )*
                    ]
                );
            )*
            map
        }
    };
}

command_handler!(
    [uow, AtomicUnitOfWork];
    {
        CreateBoard: ServiceHandler::create_board,
        EditBoard: ServiceHandler::edit_board,
        AddComment: ServiceHandler::add_comment,
        EditComment: ServiceHandler::edit_comment,
        Outbox: ServiceHandler::handle_outbox
    }
);
event_handler!(
    [uow,AtomicUnitOfWork];
    {
        BoardCreated : [EventHandler::test_event_handler,EventHandler::test_event_handler2]
    }
);
