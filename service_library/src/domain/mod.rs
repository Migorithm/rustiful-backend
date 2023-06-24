pub mod auth;
pub mod board;
pub mod builder;
pub mod commands;

use std::{any::Any, collections::VecDeque};

use serde::Serialize;

use crate::adapters::outbox::Outbox;

pub trait AnyTrait: Any {
    fn as_any(&self) -> Box<dyn Any + Send + Sync>;
}
impl<T: Any + Clone + Send + Sync> AnyTrait for T {
    // Blanket implementation
    fn as_any(&self) -> Box<dyn Any + Send + Sync> {
        Box::new(self.clone())
    }
}

pub trait Message: Any + Sync + Send + Serialize {
    fn externally_notifiable(&self) -> bool {
        false
    }

    fn get_metadata(&self) -> MessageMetadata;
    fn outbox(&self) -> Outbox {
        let metadata = self.get_metadata();
        Outbox::new(
            metadata.aggregate_id,
            metadata.topic,
            serde_json::to_string(&self).expect("Failed to serialize"),
        )
    }
}

pub struct MessageMetadata {
    pub(crate) aggregate_id: String,
    pub(crate) topic: String,
}

pub trait Aggregate: Send + Sync {
    type Event;
    fn collect_events(&mut self) -> VecDeque<Self::Event> {
        if !self.events().is_empty() {
            self.take_events()
        } else {
            VecDeque::new()
        }
    }
    fn events(&self) -> &VecDeque<Self::Event>;

    fn take_events(&mut self) -> VecDeque<Self::Event>;
    fn raise_event(&mut self, event: Self::Event);
}

trait State {
    fn state(&self) -> &str;
}
