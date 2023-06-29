use crate::{
    adapters::database::{AtomicConnection, Connection},
    domain::{
        commands::{Command, ServiceResponse},
        AnyTrait, Message,
    },
    services::handlers::init_command_handler,
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

use tokio::sync::Mutex;

use super::{
    handlers::Future,
    unit_of_work::{AtomicUnitOfWork, UnitOfWork},
};

type EventHandler<T> = Box<dyn Fn(Box<dyn Message>, T) -> Future<ServiceResponse> + Send + Sync>;
type CommandHandler<T> = HashMap<
    TypeId,
    Box<dyn Fn(Box<dyn Any + Send + Sync>, T) -> Future<ServiceResponse> + Send + Sync>,
>;

pub struct MessageBus {
    #[cfg(test)]
    pub book_keeper: i32,
    pub connection: AtomicConnection,
}

impl MessageBus {
    pub async fn new() -> Arc<Self> {
        Self {
            #[cfg(test)]
            book_keeper: 0,
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
    fn init_event_handler() -> Arc<HashMap<TypeId, Vec<EventHandler<AtomicUnitOfWork>>>> {
        // TODO As there is a host of repetitive work, this is subject to macro.
        let uow_map = HashMap::new();
        // uow_map.insert(event.type_id())
        uow_map.into()
    }

    async fn handle_event(
        &self,
        msg: Box<dyn Message>,
        uow: Arc<Mutex<UnitOfWork>>,
    ) -> ApplicationResult<()> {
        let event_handler = MessageBus::init_event_handler();
        for handler in event_handler.get(&msg.type_id()).ok_or_else(|| {
            eprintln!("Unprocessable Event Given!");
            ApplicationError::NotFound
        })? {
            handler(msg.message_clone(), uow.clone()).await?;
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

#[cfg(test)]
pub mod test_messagebus {
    use crate::domain::board::commands::CreateBoard;
    use crate::services::messagebus::MessageBus;
    use crate::utils::test_components::components::*;

    use uuid::Uuid;

    #[tokio::test]
    async fn test_message_bus_command_handling() {
        run_test(async {
            let ms = MessageBus::new().await;
            let cmd = CreateBoard {
                author: Uuid::new_v4(),
                title: "TestTitle".into(),
                content: "TestContent".into(),
                state: Default::default(),
            };
            match ms.handle(cmd).await {
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
