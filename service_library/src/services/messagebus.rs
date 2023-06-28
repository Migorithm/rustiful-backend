use crate::{
    adapters::database::AtomicConnection,
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
    marker::PhantomData,
    sync::Arc,
};
use tokio::sync::Mutex;

use super::{
    handlers::{self, Future, ServiceHandler},
    unit_of_work::{AtomicUnitOfWork, UnitOfWork},
};

#[derive(Clone)]
pub struct MessageBus<C>
where
    C: Command + AnyTrait,
{
    _phantom: PhantomData<C>,
    #[cfg(test)]
    pub book_keeper: i32,
}

impl<C> Default for MessageBus<C>
where
    C: Command + AnyTrait,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C> MessageBus<C>
where
    C: Command + AnyTrait,
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::<C>,

            #[cfg(test)]
            book_keeper: 0,
        }
    }

    pub async fn handle(
        &mut self,
        message: C,
        connection: AtomicConnection,
    ) -> ApplicationResult<ServiceResponse> {
        let uow = UnitOfWork::new(connection.clone());

        //*
        // ! We cannnot tell if handler requires or only require uow.
        // ! so it's better to take all handlers and inject dependencies so we can simply pass message
        // ! Dependency injection!

        // ? Box<dyn Fn(C)->C::Response>
        //  */
        let handler = MessageBus::<C>::init_command_handler();
        let res = handler.get(&message.type_id()).unwrap()(message.as_any(), uow.clone()).await?;

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

    fn init_command_handler(
    ) -> HashMap<TypeId, DIHandler<Box<dyn Any + Send + Sync>, AtomicUnitOfWork>> {
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
        uow_map
    }
}

type DIHandler<T, U> = Box<dyn Fn(T, U) -> Future<ServiceResponse> + Send + Sync>;

#[cfg(test)]
pub mod test_messagebus {
    use crate::adapters::database::Connection;
    use crate::domain::board::commands::CreateBoard;
    use crate::services::messagebus::MessageBus;
    use crate::utils::test_components::components::*;
    use std::marker::PhantomData;
    use uuid::Uuid;

    #[test]
    fn test_message_default() {
        let ms = MessageBus::<CreateBoard>::new();
        assert_eq!(ms._phantom, PhantomData::<CreateBoard>)
    }

    #[tokio::test]
    async fn test_message_bus_command_handling() {
        run_test(async {
            let connection = Connection::new().await.unwrap();
            let mut ms = MessageBus::<CreateBoard>::new();
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
