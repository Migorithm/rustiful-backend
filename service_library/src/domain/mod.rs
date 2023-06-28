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

pub trait MessageClone {
    fn message_clone(&self) -> Box<dyn Message>;
}

pub trait Message: Any + Sync + Send + MessageClone {
    fn externally_notifiable(&self) -> bool {
        false
    }

    fn metadata(&self) -> MessageMetadata;
    fn state(&self) -> String;
    fn outbox(&self) -> Outbox {
        let metadata = self.metadata();
        Outbox::new(metadata.aggregate_id, metadata.topic, self.state())
    }
}

pub struct MessageMetadata {
    pub(crate) aggregate_id: String,
    pub(crate) topic: String,
}

pub trait Aggregate: Send + Sync {
    fn collect_events(&mut self) -> VecDeque<Box<dyn Message>> {
        if !self.events().is_empty() {
            self.take_events()
        } else {
            VecDeque::new()
        }
    }
    fn events(&self) -> &VecDeque<Box<dyn Message>>;

    fn take_events(&mut self) -> VecDeque<Box<dyn Message>>;
    fn raise_event(&mut self, event: Box<dyn Message>);
}

trait State {
    fn state(&self) -> &str;
}
