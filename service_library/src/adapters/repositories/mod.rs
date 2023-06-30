pub mod auth_repository;
pub mod board_repository;

use crate::domain::{Aggregate, Message};
use crate::utils::ApplicationError;
use async_trait::async_trait;

use std::collections::VecDeque;
use std::marker::PhantomData;

use super::database::AtomicConnection;
use super::outbox::Outbox;

/// The abstract central source for loading past events and committing new events.
#[async_trait]
pub trait TRepository {
    type Aggregate: AsMut<Self::Aggregate> + AsRef<Self::Aggregate> + Aggregate + Send + Sync;

    fn new(connection: AtomicConnection) -> Self;

    fn get_events(&self) -> &VecDeque<Box<dyn Message>>;
    fn set_events(&mut self, events: VecDeque<Box<dyn Message>>);

    fn _collect_events(&mut self, mut aggregate: impl AsMut<Self::Aggregate> + Send + Sync) {
        self.set_events(aggregate.as_mut().collect_events())
    }
    fn _collect_outbox(&self) -> Box<dyn Iterator<Item = Outbox> + '_ + Send> {
        Box::new(
            self.get_events()
                .iter()
                .filter(|e| e.externally_notifiable())
                .map(|e| e.outbox()),
        )
    }

    async fn add(
        &mut self,
        mut aggregate: impl AsMut<Self::Aggregate> + Send + Sync,
    ) -> Result<String, ApplicationError> {
        self._collect_events(aggregate.as_mut());
        self._add(aggregate.as_mut()).await
    }

    async fn _add(
        &mut self,
        aggregate: impl AsRef<Self::Aggregate> + Send + Sync,
    ) -> Result<String, ApplicationError>;

    async fn get(&self, aggregate_id: &str) -> Result<Self::Aggregate, ApplicationError>;

    async fn update(
        &mut self,
        mut aggregate: impl AsMut<Self::Aggregate> + Send + Sync,
    ) -> Result<(), ApplicationError> {
        self._collect_events(aggregate.as_mut());
        self._update(aggregate.as_mut()).await
    }

    async fn _update(
        &mut self,
        aggregate: impl AsRef<Self::Aggregate> + Send + Sync,
    ) -> Result<(), ApplicationError>;

    fn connection(&self) -> &AtomicConnection;
}

pub struct Repository<A: Aggregate> {
    pub connection: AtomicConnection,
    pub _phantom: PhantomData<A>,
    pub events: VecDeque<Box<dyn Message>>,
}
