use crate::{
    adapters::{database::AtomicConnection, outbox::Outbox},
    domain::{
        board::commands::{AddComment, CreateBoard, EditBoard, EditComment},
        commands::{Command, ServiceResponse},
        AnyTrait, Message,
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
    handlers::{Future, ServiceHandler},
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
        // ! Dependency injection! - through boostrapping

        //  */
        let handler = MessageBus::init_command_handler();
        let res = handler.get(&message.type_id()).ok_or_else(|| {
            eprintln!("Unprocessable Command Given!");
            ApplicationError::NotFound
        })?(message.as_any(), uow.clone())
        .await?;

        let mut queue = uow.clone().lock().await._collect_events();
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
    fn init_event_handler() -> UOWMappedEventHandler<Box<dyn Message>> {
        // TODO As there is a host of repetitive work, this is subject to macro. 
        let mut uow_map: HashMap<TypeId, Vec<DIHandler<Box<dyn Message>, AtomicUnitOfWork>>> =
            HashMap::new();
        // uow_map.insert(event.type_id())
        uow_map.into()
    }

    async fn handle_event(
        &mut self,
        msg: Box<dyn Message>,
        uow: Arc<Mutex<UnitOfWork>>,
    ) -> ApplicationResult<()> {
        let event_handler = MessageBus::init_event_handler();
        for handler in event_handler.get(&msg.type_id()).ok_or_else(|| {
            eprintln!("Unprocessable Command Given!");
            ApplicationError::NotFound
        })? {
            handler(msg.message_clone(), uow.clone()).await?;
        }
        drop(uow);
        Ok(())
    }

    fn init_command_handler() -> UOWMappedHandler<Box<dyn Any + Send + Sync>> {
        // TODO As there is a host of repetitive work, this is subject to macro. 

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

type UOWMappedHandler<T> = Arc<HashMap<TypeId, DIHandler<T, AtomicUnitOfWork>>>;
type UOWMappedEventHandler<T> = Arc<HashMap<TypeId, Vec<DIHandler<T, AtomicUnitOfWork>>>>;
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
        })
        .await;
    }
}
