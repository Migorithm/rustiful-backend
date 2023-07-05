use std::sync::Arc;
use std::{collections::VecDeque, mem};

use crate::adapters::database::Executor;
use crate::domain::Message;

use crate::domain::auth::AuthAggregate;

use super::{Repository, TRepository};
use crate::utils::ApplicationError;
use async_trait::async_trait;
use tokio::sync::RwLock;

#[async_trait]
impl TRepository<AuthAggregate> for Repository<AuthAggregate> {
    fn new(executor: Arc<RwLock<Executor>>) -> Self {
        Self {
            executor,
            _phantom: Default::default(),
            events: Default::default(),
        }
    }
    fn get_events(&mut self) -> VecDeque<Box<dyn Message>> {
        mem::take(&mut self.events)
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
}
