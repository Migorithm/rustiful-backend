use tokio::sync::mpsc::error::TryRecvError;

use crate::{
    adapters::database::{AtomicContextManager, ContextManager},
    bootstrap::{CommandHandler, EventHandler},
    domain::{
        commands::{Command, ServiceResponse},
        AnyTrait, Message,
    },
    utils::{ApplicationError, ApplicationResult},
};
use std::sync::Arc;

#[cfg(test)]
use std::sync::atomic::AtomicI32;

pub struct MessageBus {
    #[cfg(test)]
    pub book_keeper: AtomicI32,

    command_handler: &'static CommandHandler<AtomicContextManager>,
    event_handler: &'static EventHandler<AtomicContextManager>,
}

impl MessageBus {
    pub fn new(
        command_handler: &'static CommandHandler<AtomicContextManager>,
        event_handler: &'static EventHandler<AtomicContextManager>,
    ) -> Arc<Self> {
        Self {
            #[cfg(test)]
            book_keeper: AtomicI32::new(0),

            command_handler,
            event_handler,
        }
        .into()
    }

    pub async fn handle<C>(&self, message: C) -> ApplicationResult<ServiceResponse>
    where
        C: Command + AnyTrait,
    {
        let (context_manager, mut event_receiver) = ContextManager::new().await;

        let res = self
            .command_handler
            .get(&message.type_id())
            .ok_or_else(|| {
                eprintln!("Unprocessable Command Given!");
                ApplicationError::CommandNotFound
            })?(message.as_any(), context_manager.clone())
        .await?;

        'event_handling_loop: loop {
            // * use of try_recv is to stop blocking it when all events are drained.
            match event_receiver.try_recv() {
                // * Logging!
                Ok(msg) => {
                    if let Err(ApplicationError::EventNotFound) =
                        self.handle_event(msg, context_manager.clone()).await
                    {
                        continue;
                    };
                }
                Err(TryRecvError::Empty) => {
                    if Arc::strong_count(&context_manager) == 1 {
                        break 'event_handling_loop;
                    } else {
                        continue;
                    }
                }
                Err(TryRecvError::Disconnected) => break 'event_handling_loop,
            };
        }
        drop(context_manager);
        Ok(res)
    }

    async fn handle_event(
        &self,
        msg: Box<dyn Message>,
        context_manager: AtomicContextManager,
    ) -> ApplicationResult<()> {
        // ! msg.topic() returns the name of event. It is crucial that it corresponds to the key registered on Event Handler.

        let handlers = self
            .event_handler
            .get(&msg.metadata().topic)
            .ok_or_else(|| {
                eprintln!("Unprocessable Event Given! {:?}", msg);
                ApplicationError::EventNotFound
            })?;

        for handler in handlers.iter() {
            match handler(msg.message_clone(), context_manager.clone()).await {
                Err(ApplicationError::StopSentinel) => {
                    eprintln!("Stop Sentinel Reached!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error Occurred While Handling Event! Error:{}", err);
                }
                Ok(_val) => {
                    println!("Event Handling Succeeded!");
                }
            };

            #[cfg(test)]
            {
                self.book_keeper
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
        drop(context_manager);
        Ok(())
    }
}

// ----------------------------------------------------------------------- //
#[cfg(test)]
pub mod test_messagebus {
    use std::str::FromStr;
    use std::sync::Arc;

    use crate::adapters::database::Executor;
    use crate::adapters::repositories::{Repository, TRepository};
    use crate::bootstrap::{connection_pool, Boostrap};
    use crate::domain::board::commands::{AddComment, CreateBoard, EditBoard};
    use crate::domain::board::BoardAggregate;
    use crate::domain::commands::ServiceResponse;

    use crate::utils::test_components::components::*;

    use tokio::sync::RwLock;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_message_bus_create_board() {
        run_test(async {
            let pool = connection_pool().await;
            let executor = Arc::new(RwLock::new(Executor::new(pool)));
            let ms = Boostrap::message_bus().await;
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

                let repo = Repository::<BoardAggregate>::new(executor);
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
            let pool = connection_pool().await;
            let executor = Arc::new(RwLock::new(Executor::new(pool)));
            let ms = Boostrap::message_bus().await;
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
                let repo = Repository::<BoardAggregate>::new(executor);
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
            let pool = connection_pool().await;
            let executor = Arc::new(RwLock::new(Executor::new(pool)));
            let ms = Boostrap::message_bus().await;
            let create_cmd = CreateBoard {
                author: Uuid::new_v4(),
                title: "TestTitle".into(),
                content: "TestContent".into(),
                state: Default::default(),
            };

            let id = match ms.handle(create_cmd).await {
                Ok(ServiceResponse::String(id)) => id,
                _ => panic!("Failed!"),
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

                let repo = Repository::<BoardAggregate>::new(executor);
                let Ok(aggregate) = repo.get(&id).await else{
                        panic!("Something wrong")
                };

                assert_eq!(aggregate.comments.len(), 1);
            }
        })
        .await;
    }
}
