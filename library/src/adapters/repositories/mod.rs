pub mod auth_repository;
pub mod board_repository;

use crate::domain::{Aggregate, Message};

use async_trait::async_trait;
use tokio::sync::RwLock;

use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;

use super::database::Executor;

/// The abstract central source for loading past events and committing new events.
#[async_trait]
pub trait TRepository {
    fn new(executor: Arc<RwLock<Executor>>) -> Self;

    fn get_events(&mut self) -> VecDeque<Box<dyn Message>>;
    fn set_events(&mut self, events: VecDeque<Box<dyn Message>>);


}

pub struct Repository<A: Aggregate> {
    pub executor: Arc<RwLock<Executor>>,
    pub _phantom: PhantomData<A>,
    pub events: VecDeque<Box<dyn Message>>,
}


impl<A:Aggregate> TRepository for Repository<A>{
    fn get_events(&mut self) -> VecDeque<Box<dyn Message> > {
        mem::take(&mut self.events)
    }
    fn new(executor:Arc<RwLock<Executor> >) -> Self {
        Self { executor, _phantom: PhantomData, events: Default::default() }
    }
    fn set_events(&mut self,events:VecDeque<Box<dyn Message> >) {
        self.events = events
    }
}