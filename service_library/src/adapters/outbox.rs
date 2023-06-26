use std::{any::Any, sync::Arc};

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    domain::{
        auth::{self, events::AuthEvent},
        board::{self, events::BoardEvent},
        commands::{Command, ServiceResponse},
        AnyTrait,
    },
    services::{
        handlers::{Future, Handler},
        unit_of_work::UnitOfWork,
    },
    utils::{ApplicationError, ApplicationResult},
};

use super::database::AtomicConnection;

#[derive(Debug, Clone)]
pub struct Outbox {
    id: Uuid,
    aggregate_id: String,
    topic: String,
    state: String,
    processed: bool,
    create_dt: DateTime<Utc>,
}

impl Outbox {
    pub fn new(aggregate_id: String, topic: String, state: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            topic,
            state,
            processed: false,
            create_dt: Default::default(),
        }
    }
    pub fn convert_event(&self) -> Box<dyn Any + Send + Sync> {
        match self.topic.as_str() {
            board::events::TOPIC => serde_json::from_str::<BoardEvent>(self.state.as_str())
                .unwrap()
                .as_any(),

            auth::events::TOPIC => serde_json::from_str::<AuthEvent>(self.state.as_str())
                .unwrap()
                .as_any(),
            _ => panic!("Not served!"),
        }
    }
    pub fn tag_processed(&mut self) {
        self.processed = true
    }

    pub async fn add(connection: AtomicConnection, outboxes: Vec<Self>) -> ApplicationResult<()> {
        for ob in outboxes {
            sqlx::query_as!(
                Self,
                "INSERT INTO service_outbox 
            (id, aggregate_id, topic, state, processed, create_dt) VALUES 
            ($1, $2, $3, $4, $5, $6)
            ",
                ob.id,
                ob.aggregate_id,
                ob.topic,
                ob.state,
                ob.processed,
                ob.create_dt,
            )
            .execute(connection.write().await.connection())
            .await
            .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;
        }
        Ok(())
    }
    pub async fn get(connection: AtomicConnection) -> ApplicationResult<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT * FROM service_outbox WHERE processed = $1"#,
            false
        )
        .fetch_all(&connection.read().await.pool)
        .await
        .map_err(|err| {
            eprintln!("{}", err);
            ApplicationError::DatabaseConnectionError(Box::new(err))
        })
    }
    pub async fn update(&self, connection: &mut AtomicConnection) -> ApplicationResult<()> {
        sqlx::query_as!(
            Self,
            r#" 
                UPDATE service_outbox SET 
                processed =$1
                WHERE id = $2
            "#,
            true,
            self.id,
        )
        .execute(connection.write().await.connection())
        .await
        .map_err(|err| {
            eprintln!("{}", err);
            ApplicationError::DatabaseConnectionError(Box::new(err))
        })?;
        Ok(())
    }
}

impl Command for Outbox {
    type Handler = OutboxHandler;
    type Response = ServiceResponse;
}

pub struct OutboxHandler;
impl Handler for OutboxHandler {
    type Command = Outbox;
    type Response = ServiceResponse;
    fn execute(outbox: Self::Command, uow: Arc<Mutex<UnitOfWork>>) -> Future<Self::Response> {
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

#[cfg(test)]
mod test_outbox {
    use core::panic;

    use uuid::Uuid;

    use crate::domain::board::events::BoardEvent;
    use crate::domain::commands::ServiceResponse;
    use crate::services::messagebus::MessageBus;
    use crate::utils::test_components::components::*;
    use crate::{
        adapters::{
            database::{AtomicConnection, Connection},
            outbox::Outbox,
            repositories::TRepository,
        },
        domain::{board::entity::BoardState, commands::ApplicationCommand},
        services::{
            handlers::{Handler, ServiceHandler},
            unit_of_work::UnitOfWork,
        },
    };

    async fn outbox_setup(connection: AtomicConnection) {
        let cmd = ApplicationCommand::CreateBoard {
            author: Uuid::new_v4(),
            title: "Title!".to_string(),
            content: "Content".to_string(),
            state: BoardState::Published,
        };

        let uow = UnitOfWork::new(connection.clone());
        match ServiceHandler::execute(cmd, uow.clone()).await {
            Err(err) => '_fail_case: {
                panic!("Service Handling Failed! {}", err)
            }
            Ok(response) => '_test: {
                let uow = UnitOfWork::new(connection);
                let ServiceResponse::String(id) = response else{
                    panic!("Wrong Variant");
                };

                if let Err(err) = uow.lock().await.boards.get(&id).await {
                    panic!("Fetching newly created object failed! : {}", err);
                };
            }
        }
    }

    #[tokio::test]
    async fn test_create_board_leaves_outbox() {
        run_test(|| {
            Box::pin(async {
                let connection = Connection::new().await.unwrap();
                outbox_setup(connection.clone()).await;

                '_test_case: {
                    match Outbox::get(connection.clone()).await {
                        Err(err) => {
                            eprintln!("{}", err);
                            panic!("Test Failed! Outbox Not Stored!")
                        }
                        Ok(val) => {
                            println!("{:?}", val);
                            println!("Outbox stored successfully!")
                        }
                    }
                }
            })
        })
        .await;
    }

    #[tokio::test]
    async fn test_convert_event() {
        run_test(|| {
            Box::pin(async {
                let connection = Connection::new().await.unwrap();
                outbox_setup(connection.clone()).await;

                '_test_case: {
                    let vec_of_outbox = Outbox::get(connection.clone()).await.unwrap();

                    assert_eq!(vec_of_outbox.len(), 1);
                    let event = vec_of_outbox.get(0).unwrap().convert_event();
                    assert!(event.is::<BoardEvent>());

                    let converted = *event.downcast::<BoardEvent>().unwrap();
                    match converted {
                        BoardEvent::Created { .. } => {
                            println!("Success!")
                        }
                        _ => {
                            panic!("Failed!")
                        }
                    };
                }
            })
        })
        .await
    }

    #[tokio::test]
    async fn test_outbox_event_handled_by_messagebus() {
        run_test(|| {
            Box::pin(async {
                let connection = Connection::new().await.unwrap();
                outbox_setup(connection.clone()).await;

                '_test_case: {
                    let mut bus = MessageBus::<Outbox>::new();

                    for e in Outbox::get(connection.clone()).await.unwrap() {
                        match bus.handle(e, connection.clone()).await {
                            Ok(var) => {
                                assert_eq!(var.len(), 1);
                            }
                            Err(_) => panic!("Failed!"),
                        }
                    }

                    // TODO where does the processed tag get modifeid?
                    let boxes = Outbox::get(connection.clone()).await.unwrap();
                    assert!(boxes.is_empty());
                }
            })
        })
        .await;
    }
}
