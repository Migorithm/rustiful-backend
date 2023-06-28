use chrono::{DateTime, Utc};

use serde::{de::IntoDeserializer, Deserialize};
use uuid::Uuid;

use crate::{
    domain::{
        auth::events::{AccountCreated, AccountUpdated},
        board::events::{BoardCommentAdded, BoardCreated, BoardUpdated},
        commands::Command,
        Message,
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
    pub fn convert_event(&self) -> Box<dyn Message> {
        match self.topic.as_str() {
            "BoardCreated" => serde_json::from_str::<BoardCreated>(self.state.as_str())
                .unwrap()
                .message_clone(),
            "BoardUpdated" => serde_json::from_str::<BoardUpdated>(self.state.as_str())
                .unwrap()
                .message_clone(),
            "BoardCommentAdded" => serde_json::from_str::<BoardCommentAdded>(self.state.as_str())
                .unwrap()
                .message_clone(),
            "AccountCreated" => serde_json::from_str::<AccountCreated>(self.state.as_str())
                .unwrap()
                .message_clone(),
            "AccountUpdated" => serde_json::from_str::<AccountUpdated>(self.state.as_str())
                .unwrap()
                .message_clone(),
            _ => panic!("Wrong Message!"),
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

impl Command for Outbox {}
