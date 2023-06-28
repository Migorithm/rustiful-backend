use crate::{
    adapters::{database::AtomicConnection, outbox::Outbox},
    domain::{
        auth::events::AuthEvent,
        board::{
            commands::{AddComment, CreateBoard, EditBoard, EditComment},
            events::BoardEvent,
        },
        commands::{Command, ServiceResponse},
        AnyTrait,
    },
    utils::{ApplicationError, ApplicationResult},
};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::Mutex;

use super::{
    handlers::{self, Future, ServiceHandler},
    unit_of_work::{AtomicUnitOfWork, UnitOfWork},
};

#[derive(Clone)]
pub struct MessageBus {
    #[cfg(test)]
    pub book_keeper: i32,
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            #[cfg(test)]
            book_keeper: 0,
        }
    }

    pub async fn handle<C>(
        &mut self,
        message: C,
        connection: AtomicConnection,
    ) -> ApplicationResult<ServiceResponse>
    where
        C: Command + AnyTrait,
    {
        let uow = UnitOfWork::new(connection.clone());

        //*
        // ! We cannnot tell if handler requires or only require uow.
        // ! so it's better to take all handlers and inject dependencies so we can simply pass message
        // ! Dependency injection!
        
        //  */
        let handler = MessageBus::init_command_handler();
        let res = handler.get(&message.type_id()).ok_or_else(|| {
            eprintln!("Unprocessable Command Given!");
            ApplicationError::NotFound
        })?(message.as_any(), uow.clone())
        .await?;

        // message.handle(uow.clone()).await;
        let mut queue = uow.clone().lock().await._collect_events();

        // TODO Handle Command First and loop event?

        while let Some(msg) = queue.pop_front() {
            // * Logging!

            match self.handle_event(msg, uow.clone()).await {
                Err(ApplicationError::StopSentinel) => {
                    eprintln!("Stop Sentinel Reached!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error Occurred While Handling Event! Error:{}", err);
                }
                Ok(_) => {
                    println!("Event Handling Succeeded!")
                }
            }

            for new_event in uow.clone().lock().await._collect_events() {
                queue.push_back(new_event);
            }
        }
        drop(uow);
        Ok(res)
    }

    async fn handle_event(
        &mut self,
        msg: Box<dyn Any + Send + Sync>,
        uow: Arc<Mutex<UnitOfWork>>,
    ) -> ApplicationResult<()> {
        if msg.is::<BoardEvent>() {
            let event: BoardEvent = *msg.downcast::<BoardEvent>().unwrap();
            match event {
                BoardEvent::Created { .. } => {
                    for handler in handlers::BOARD_CREATED_EVENT_HANDLERS.iter() {
                        handler(event.clone(), uow.clone()).await?;
                    }
                }
                BoardEvent::Updated { .. } => {
                    for handler in handlers::BOARD_UPDATED_EVENT_HANDLERS.iter() {
                        handler(event.clone(), uow.clone()).await?;
                    }
                }
                BoardEvent::CommentAdded { .. } => {
                    for handler in handlers::COMMENT_ADDED_EVENT_HANDLERS.iter() {
                        handler(event.clone(), uow.clone()).await?;
                    }
                }
            };
            #[cfg(test)]
            {
                self.book_keeper += 1
            }
        } else if msg.is::<AuthEvent>() {
            let event: AuthEvent = *msg.downcast::<AuthEvent>().unwrap();
            match event {
                AuthEvent::Created { .. } => {
                    for handler in handlers::ACCOUNT_CREATED_EVENT_HANDLERS.iter() {
                        handler(event.clone(), uow.clone()).await?;
                    }
                }
                AuthEvent::Updated { .. } => {
                    for handler in handlers::ACCOUNT_UPDATED_EVENT_HANDLERS.iter() {
                        handler(event.clone(), uow.clone()).await?;
                    }
                }
            };
            #[cfg(test)]
            {
                self.book_keeper += 1
            }
        } else {
            Err(ApplicationError::InExecutableEvent)?
        }

        drop(uow);
        Ok(())
    }

    fn init_command_handler() -> UOWMappedHandler {
        let mut uow_map: HashMap<TypeId, DIHandler<Box<dyn Any + Send + Sync>, AtomicUnitOfWork>> =
            HashMap::new();
        uow_map.insert(
            TypeId::of::<CreateBoard>(),
            Box::new(
                |c: Box<dyn Any + Send + Sync>, uow: AtomicUnitOfWork| -> Future<ServiceResponse> {
                    ServiceHandler::create_board(*c.downcast::<CreateBoard>().unwrap(), uow)
                },
            ),
        );
        uow_map.insert(
            TypeId::of::<EditBoard>(),
            Box::new(
                |c: Box<dyn Any + Send + Sync>, uow: AtomicUnitOfWork| -> Future<ServiceResponse> {
                    ServiceHandler::edit_board(*c.downcast::<EditBoard>().unwrap(), uow)
                },
            ),
        );
        uow_map.insert(
            TypeId::of::<AddComment>(),
            Box::new(
                |c: Box<dyn Any + Send + Sync>, uow: AtomicUnitOfWork| -> Future<ServiceResponse> {
                    ServiceHandler::add_comment(*c.downcast::<AddComment>().unwrap(), uow)
                },
            ),
        );
        uow_map.insert(
            TypeId::of::<EditComment>(),
            Box::new(
                |c: Box<dyn Any + Send + Sync>, uow: AtomicUnitOfWork| -> Future<ServiceResponse> {
                    ServiceHandler::edit_comment(*c.downcast::<EditComment>().unwrap(), uow)
                },
            ),
        );
        uow_map.insert(
            TypeId::of::<Outbox>(),
            Box::new(
                |c: Box<dyn Any + Send + Sync>, uow: AtomicUnitOfWork| -> Future<ServiceResponse> {
                    ServiceHandler::handle_outbox(*c.downcast::<Outbox>().unwrap(), uow)
                },
            ),
        );
        uow_map.into()
    }
}

type UOWMappedHandler =
    Arc<HashMap<TypeId, DIHandler<Box<dyn Any + Send + Sync>, AtomicUnitOfWork>>>;
type DIHandler<T, U> = Box<dyn Fn(T, U) -> Future<ServiceResponse> + Send + Sync>;

#[cfg(test)]
pub mod test_messagebus {
    use crate::adapters::database::Connection;
    use crate::domain::board::commands::CreateBoard;
    use crate::services::messagebus::MessageBus;
    use crate::utils::test_components::components::*;

    use uuid::Uuid;

    #[tokio::test]
    async fn test_message_bus_command_handling() {
        run_test(async {
            let connection = Connection::new().await.unwrap();
            let mut ms = MessageBus::new();
            let cmd = CreateBoard {
                author: Uuid::new_v4(),
                title: "TestTitle".into(),
                content: "TestContent".into(),
                state: Default::default(),
            };
            match ms.handle(cmd, connection).await {
                Ok(res) => {
                    println!("{:?}", res)
                }
                Err(err) => {
                    eprintln!("{}", err);
                    panic!("Test Failed!")
                }
            };

            assert_eq!(ms.book_keeper, 1);
        })
        .await;
    }
}
