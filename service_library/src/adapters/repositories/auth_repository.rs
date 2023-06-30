use std::collections::VecDeque;

use crate::{adapters::database::AtomicConnection, domain::Message};

use crate::domain::auth::AuthAggregate;

use super::{Repository, TRepository};
use crate::utils::ApplicationError;
use async_trait::async_trait;

#[async_trait]
impl TRepository<AuthAggregate> for Repository<AuthAggregate> {
    fn new(connection: AtomicConnection) -> Self {
        Self {
            connection,
            _phantom: Default::default(),
            events: Default::default(),
        }
    }
    fn get_events(&self) -> &VecDeque<Box<dyn Message>> {
        &self.events
    }
    fn set_events(&mut self, events: VecDeque<Box<dyn Message>>) {
        self.events = events
    }

    async fn _add(&mut self, _aggregate: &AuthAggregate) -> Result<String, ApplicationError> {
        unimplemented!()
    }

    async fn get(&self, _aggregate_id: &str) -> Result<AuthAggregate, ApplicationError> {
        unimplemented!()
    }

    async fn _update(&mut self, _aggregate: &AuthAggregate) -> Result<(), ApplicationError> {
        unimplemented!()
    }

    fn connection(&self) -> &AtomicConnection {
        &self.connection
    }
}
