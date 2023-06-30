use std::collections::VecDeque;

use crate::{adapters::database::AtomicConnection, domain::Message};

use crate::domain::auth::AuthAggregate;

use crate::utils::ApplicationError;
use async_trait::async_trait;

use super::{Repository, TRepository};

#[async_trait]
impl TRepository for Repository<AuthAggregate> {
    type Aggregate = AuthAggregate;

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

    async fn _add(
        &mut self,
        _aggregate: impl AsRef<Self::Aggregate> + Send + Sync,
    ) -> Result<String, ApplicationError> {
        unimplemented!()
    }

    async fn get(&mut self, _aggregate_id: &str) -> Result<Self::Aggregate, ApplicationError> {
        unimplemented!()
    }

    async fn _update(
        &mut self,
        _aggregate: impl AsRef<Self::Aggregate> + Send + Sync,
    ) -> Result<(), ApplicationError> {
        unimplemented!()
    }

    fn connection(&self) -> &AtomicConnection {
        &self.connection
    }
}
