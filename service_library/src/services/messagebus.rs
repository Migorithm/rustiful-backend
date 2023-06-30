use crate::{
    adapters::database::{AtomicConnection, Connection},
    domain::{
        commands::{Command, ServiceResponse},
        AnyTrait, Message,
    },
    services::handlers::{init_command_handler, init_event_handler},
    utils::{ApplicationError, ApplicationResult},
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{
        atomic::AtomicPtr,
        atomic::Ordering::{Acquire, Release},
        Arc,
    },
};

#[cfg(test)]
use std::sync::atomic::AtomicI32;

use super::{
    handlers::Future,
    unit_of_work::{AtomicUnitOfWork, UnitOfWork},
};

type EventHandler<T> =
    HashMap<String, Vec<Box<dyn Fn(Box<dyn Message>, T) -> Future<ServiceResponse> + Send + Sync>>>;
type CommandHandler<T> = HashMap<
    TypeId,
    Box<dyn Fn(Box<dyn Any + Send + Sync>, T) -> Future<ServiceResponse> + Send + Sync>,
>;

pub struct MessageBus {
    #[cfg(test)]
    pub book_keeper: AtomicI32,
    pub connection: AtomicConnection,
}

impl MessageBus {
    pub async fn new() -> Arc<Self> {
        Self {
            #[cfg(test)]
            book_keeper: AtomicI32::new(0),
            connection: Connection::new()
                .await
                .expect("Connection Creation Failed!"),
        }
        .into()
    }

    pub async fn handle<C>(&self, message: C) -> ApplicationResult<ServiceResponse>
    where
        C: Command + AnyTrait,
    {
        let uow = UnitOfWork::new(self.connection.clone());

        //*
        // ! We cannnot tell if handler requires or only require uow.
        // ! so it's better to take all handlers and inject dependencies so we can simply pass message
        // ! Dependency injection! - through boostrapping
        //  */
        let res = command_handler().get(&message.type_id()).ok_or_else(|| {
            eprintln!("Unprocessable Command Given!");
            ApplicationError::NotFound
        })?(message.as_any(), uow.clone())
        .await?;

        let mut queue = uow.clone().write().await._collect_events();
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

            for new_event in uow.clone().write().await._collect_events() {
                queue.push_back(new_event);
            }
        }
        drop(uow);
        Ok(res)
    }

    async fn handle_event(
        &self,
        msg: Box<dyn Message>,
        uow: AtomicUnitOfWork,
    ) -> ApplicationResult<()> {
        let event_handler = event_handler();
        for handler in event_handler.get(msg.topic()).ok_or_else(|| {
            eprintln!("Unprocessable Event Given!");
            ApplicationError::NotFound
        })? {
            handler(msg.message_clone(), uow.clone()).await?;

            #[cfg(test)]
            {
                self.book_keeper
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
        drop(uow);
        Ok(())
    }
}

fn command_handler() -> &'static CommandHandler<AtomicUnitOfWork> {
    static PTR: AtomicPtr<CommandHandler<AtomicUnitOfWork>> = AtomicPtr::new(std::ptr::null_mut());
    let mut p = PTR.load(Acquire);

    if p.is_null() {
        p = Box::into_raw(Box::new(init_command_handler()));
        if let Err(e) = PTR.compare_exchange(std::ptr::null_mut(), p, Release, Acquire) {
            // Safety: p comes from Box::into_raw right above
            // and wasn't whared with any other thread
            drop(unsafe { Box::from_raw(p) });
            p = e;
        }
    }
    // Safety: p is not null and points to a properly initialized value
    unsafe { &*p }
}

fn event_handler() -> &'static EventHandler<AtomicUnitOfWork> {
    static PTR: AtomicPtr<EventHandler<AtomicUnitOfWork>> = AtomicPtr::new(std::ptr::null_mut());
    let mut p = PTR.load(Acquire);

    if p.is_null() {
        p = Box::into_raw(Box::new(init_event_handler()));
        if let Err(e) = PTR.compare_exchange(std::ptr::null_mut(), p, Release, Acquire) {
            // Safety: p comes from Box::into_raw right above
            // and wasn't whared with any other thread
            drop(unsafe { Box::from_raw(p) });
            p = e;
        }
    }
    // Safety: p is not null and points to a properly initialized value
    unsafe { &*p }
}

// ----------------------------------------------------------------------- //
#[cfg(test)]
pub mod test_messagebus {
    use std::str::FromStr;

    use crate::adapters::repositories::{Repository, TRepository};
    use crate::domain::board::commands::{AddComment, CreateBoard, EditBoard};
    use crate::domain::board::BoardAggregate;
    use crate::domain::commands::ServiceResponse;
    use crate::services::messagebus::MessageBus;
    use crate::utils::test_components::components::*;

    use uuid::Uuid;

    #[tokio::test]
    async fn test_message_bus_create_board() {
        run_test(async {
            let ms = MessageBus::new().await;
            let cmd = CreateBoard {
                author: Uuid::new_v4(),
                title: "TestTitle".into(),
                content: "TestContent".into(),
                state: Default::default(),
            };

            '_test_code: {
                let Ok(ServiceResponse::String(var)) = ms.handle(cmd).await else{
                panic!("Test Failed!")
                };

                let repo = Repository::<BoardAggregate>::new(ms.connection.clone());
                match repo.get(&var).await {
                    Ok(_created_board) => println!("Success!"),
                    Err(err) => {
                        eprintln!("{}", err);
                        panic!("Something wrong")
                    }
                };
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_message_bus_edit_board() {
        run_test(async {
            let ms = MessageBus::new().await;
            let create_cmd = CreateBoard {
                author: Uuid::new_v4(),
                title: "TestTitle".into(),
                content: "TestContent".into(),
                state: Default::default(),
            };
            let Ok(ServiceResponse::String(id)) =  ms.handle(create_cmd).await else {
                panic!("There must be!")
            };

            '_test_code: {
                let edit_cmd = EditBoard {
                    id: Uuid::from_str(&id).unwrap(),
                    title: Some("TestTitle2".into()),
                    content: Some("ChangedContent".into()),
                    state: Default::default(),
                };
                let Ok(ServiceResponse::Empty(())) = ms.handle(edit_cmd).await else{
                    panic!("There must be!")
                };
                let repo = Repository::<BoardAggregate>::new(ms.connection.clone());
                let Ok(aggregate) = repo.get(&id).await else{
                        panic!("Something wrong")
                };

                assert_eq!(&aggregate.board.title, "TestTitle2");
                assert_eq!(&aggregate.board.content, "ChangedContent");
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_message_bus_add_comment() {
        run_test(async {
            let ms = MessageBus::new().await;
            let create_cmd = CreateBoard {
                author: Uuid::new_v4(),
                title: "TestTitle".into(),
                content: "TestContent".into(),
                state: Default::default(),
            };
            let Ok(ServiceResponse::String(id)) =  ms.handle(create_cmd).await else {
                panic!("There must be!")
            };

            '_test_code: {
                let add_comment_cmd = AddComment {
                    board_id: Uuid::from_str(&id).unwrap(),
                    author: Uuid::new_v4(),
                    content: "Good Content!".into(),
                };
                let Ok(ServiceResponse::Empty(())) = ms.handle(add_comment_cmd).await else{
                    panic!("Test Failed!")
                };

                let repo = Repository::<BoardAggregate>::new(ms.connection.clone());
                let Ok(aggregate) = repo.get(&id).await else{
                        panic!("Something wrong")
                };

                assert_eq!(aggregate.comments.len(), 1);
            }
        })
        .await;
    }
}
