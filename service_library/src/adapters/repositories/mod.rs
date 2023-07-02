pub mod auth_repository;
pub mod board_repository;

use crate::domain::{Aggregate, Message};
use crate::utils::ApplicationError;
use async_trait::async_trait;
use tokio::sync::RwLock;

use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::Arc;

use super::database::Executor;
use super::outbox::Outbox;

/// The abstract central source for loading past events and committing new events.
#[async_trait]
pub trait TRepository<A: Aggregate + 'static> {
    fn new(executor: Arc<RwLock<Executor>>) -> Self;

    fn get_events(&self) -> &VecDeque<Box<dyn Message>>;
    fn set_events(&mut self, events: VecDeque<Box<dyn Message>>);

    fn _collect_events(&mut self, aggregate: &mut A) {
        self.set_events(aggregate.collect_events())
    }
    fn _collect_outbox(&self) -> Box<dyn Iterator<Item = Outbox> + '_ + Send> {
        Box::new(
            self.get_events()
                .iter()
                .filter(|e| e.externally_notifiable())
                .map(|e| e.outbox()),
        )
    }
    async fn add(&mut self, aggregate: &mut A) -> Result<String, ApplicationError> {
        self._collect_events(aggregate);
        self._add(aggregate).await
    }

    async fn _add(&mut self, aggregate: &A) -> Result<String, ApplicationError>;

    async fn get(&self, aggregate_id: &str) -> Result<A, ApplicationError>;

    async fn update(&mut self, aggregate: &mut A) -> Result<(), ApplicationError> {
        self._collect_events(aggregate);
        self._update(aggregate).await
    }

    async fn _update(&mut self, aggregate: &A) -> Result<(), ApplicationError>;
}

pub struct Repository<A: Aggregate> {
    pub executor: Arc<RwLock<Executor>>,
    pub _phantom: PhantomData<A>,
    pub events: VecDeque<Box<dyn Message>>,
}
