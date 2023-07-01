use crate::{
    adapters::database::{AtomicConnection, Connection},
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
    pub connection: AtomicConnection,
    command_handler: &'static CommandHandler<AtomicConnection>,
    event_handler: &'static EventHandler<AtomicConnection>,
}

impl MessageBus {
    pub async fn new(
        command_handler: &'static CommandHandler<AtomicConnection>,
        event_handler: &'static EventHandler<AtomicConnection>,
    ) -> Arc<Self> {
        Self {
            #[cfg(test)]
            book_keeper: AtomicI32::new(0),
            connection: Connection::new()
                .await
                .expect("Connection Creation Failed!"),
            command_handler,
            event_handler,
        }
        .into()
    }

    pub async fn handle<C>(&self, message: C) -> ApplicationResult<ServiceResponse>
    where
        C: Command + AnyTrait,
    {
        let res = self
            .command_handler
            .get(&message.type_id())
            .ok_or_else(|| {
                eprintln!("Unprocessable Command Given!");
                ApplicationError::NotFound
            })?(message.as_any(), self.connection.clone())
        .await?;

        let mut queue = self.connection.write().await.events();
        while let Some(msg) = queue.pop_front() {
            // * Logging!

            match self.handle_event(msg, self.connection.clone()).await {
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

            for new_event in self.connection.write().await.events() {
                queue.push_back(new_event);
            }
        }

        Ok(res)
    }

    async fn handle_event(
        &self,
        msg: Box<dyn Message>,
        conn: AtomicConnection,
    ) -> ApplicationResult<()> {
        // ! msg.topic() returns the name of event. It is crucial that it corresponds to the key registered on Event Handler.
        for handler in self
            .event_handler
            .get(&msg.metadata().topic)
            .ok_or_else(|| {
                eprintln!("Unprocessable Event Given!");
                ApplicationError::NotFound
            })?
        {
            handler(msg.message_clone(), conn.clone()).await?;

            #[cfg(test)]
            {
                self.book_keeper
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
        drop(conn);
        Ok(())
    }
}

// ----------------------------------------------------------------------- //
#[cfg(test)]
pub mod test_messagebus {
    use std::str::FromStr;

    use crate::adapters::repositories::{Repository, TRepository};
    use crate::bootstrap::Boostrap;
    use crate::domain::board::commands::{AddComment, CreateBoard, EditBoard};
    use crate::domain::board::BoardAggregate;
    use crate::domain::commands::ServiceResponse;

    use crate::utils::test_components::components::*;

    use uuid::Uuid;

    #[tokio::test]
    async fn test_message_bus_create_board() {
        run_test(async {
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
