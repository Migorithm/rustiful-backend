use std::sync::Arc;

use chrono::{DateTime, Utc};

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    domain::{board::events::BoardCreated, commands::Command, Message},
    utils::{ApplicationError, ApplicationResult},
};

use super::database::Executor;

#[derive(Debug, Clone)]
pub struct Outbox {
    id: Uuid,
    aggregate_id: String,
    topic: String,
    state: String,
    processed: bool,
    create_dt: DateTime<Utc>,
}

macro_rules! convert_event {
    ( $obj:expr, $( $type: ty ), * ) => {
        match $obj.topic.as_str() {
          $(stringify!($type)=> serde_json::from_str::<$type>($obj.state.as_str()).expect("Given type not deserializable!").message_clone() ,)*
          _ => {
                panic!("Such event not allowed to process through outbox.");
          }
        }
    };
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
    pub fn convert_event(&self) -> Box<dyn Message> {
        // convert event. it takes outbox reference and target type that is to be deserialized.
        // you can insert any number of desired type as long as it is outboxable type.
        convert_event!(self, BoardCreated)
    }
    pub fn tag_processed(&mut self) {
        self.processed = true
    }

    pub async fn add(
        connection: Arc<RwLock<Executor>>,
        outboxes: Vec<Self>,
    ) -> ApplicationResult<()> {
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
    pub async fn get(executor: Arc<RwLock<Executor>>) -> ApplicationResult<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT * FROM service_outbox WHERE processed = $1"#,
            false
        )
        .fetch_all(executor.read().await.pool)
        .await
        .map_err(|err| {
            eprintln!("{}", err);
            ApplicationError::DatabaseConnectionError(Box::new(err))
        })
    }
    pub async fn update(&self, executor: Arc<RwLock<Executor>>) -> ApplicationResult<()> {
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
        .execute(executor.write().await.transaction())
        .await
        .map_err(|err| {
            eprintln!("{}", err);
            ApplicationError::DatabaseConnectionError(Box::new(err))
        })?;
        Ok(())
    }
}

impl Command for Outbox {}
