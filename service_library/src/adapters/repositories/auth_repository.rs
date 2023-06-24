use std::collections::VecDeque;

use crate::adapters::database::AtomicConnection;
use crate::adapters::repository::{Connection, Repository, TRepository};

use crate::domain::auth::AuthAggregate;

use crate::domain::auth::events::AuthEvent;
use crate::utils::ApplicationError;
use async_trait::async_trait;

#[async_trait]
impl TRepository for Repository<AuthAggregate, AuthEvent> {
    type Aggregate = AuthAggregate;
    type Event = AuthEvent;

    fn new(connection: Connection) -> Self {
        Self {
            connection,
            _phantom: Default::default(),
            events: Default::default(),
        }
    }
    fn get_events(&self) -> &VecDeque<Self::Event> {
        &self.events
    }
    fn set_events(&mut self, events: VecDeque<Self::Event>) {
        self.events = events
    }

    async fn _add(
        &mut self,
        aggregate: impl AsRef<Self::Aggregate> + Send + Sync,
    ) -> Result<String, ApplicationError> {
        unimplemented!()
    }

    async fn get(&mut self, aggregate_id: &str) -> Result<Self::Aggregate, ApplicationError> {
        unimplemented!()
    }

    async fn _update(
        &mut self,
        aggregate: impl AsRef<Self::Aggregate> + Send + Sync,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }

    fn connection(&self) -> &AtomicConnection {
        &self.connection
    }
}
