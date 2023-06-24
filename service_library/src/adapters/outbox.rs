use std::any::Any;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    domain::{
        auth::{self, events::AuthEvent},
        board::{self, events::BoardEvent},
        AnyTrait,
    },
    utils::{ApplicationError, ApplicationResult},
};

use super::database::AtomicConnection;

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
    pub fn convert_event(&mut self) -> Box<dyn Any + Send + Sync> {
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
}

#[cfg(test)]
mod test_outbox {
    use dotenv::dotenv;
    use uuid::Uuid;

    use crate::{
        adapters::{database::Connection, outbox::Outbox, repository::TRepository},
        domain::{board::entity::BoardState, commands::ApplicationCommand},
        services::{
            handlers::{Handler, ServiceHandler},
            unit_of_work::UnitOfWork,
        },
    };

    #[tokio::test]
    async fn test_create_board_leaves_outbox() {
        dotenv().unwrap();

        let cmd = ApplicationCommand::CreateBoard {
            author: Uuid::new_v4(),
            title: "Title!".to_string(),
            content: "Content".to_string(),
            state: BoardState::Published,
        };

        let connection = Connection::new().await.unwrap();
        let uow = UnitOfWork::new(connection.clone());
        match ServiceHandler::execute(cmd, uow.clone()).await {
            Err(err) => '_fail_case: {
                panic!("Service Handling Failed! {}", err)
            }
            Ok(id) => '_test: {
                let uow = UnitOfWork::new(connection.clone());
                if let Err(err) = uow.lock().await.boards.get(&id).await {
                    panic!("Fetching newly created object failed! : {}", err);
                };
            }
        }

        '_test_case: {
            if let Err(err) = Outbox::get(connection.clone()).await {
                eprintln!("{}", err);
                panic!("Test Failed! Outbox Not Stored!")
            } else {
                println!("Outbox stored successfully!")
            }
        }
    }
}
