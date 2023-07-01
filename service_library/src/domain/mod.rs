pub mod auth;
pub mod board;
pub mod builder;
pub mod commands;

use std::{any::Any, collections::VecDeque, fmt::Debug};

use downcast_rs::{impl_downcast, Downcast};

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

pub trait Message: Sync + Send + Any + Downcast {
    fn externally_notifiable(&self) -> bool {
        false
    }

    fn metadata(&self) -> MessageMetadata;
    fn outbox(&self) -> Outbox {
        let metadata = self.metadata();
        Outbox::new(metadata.aggregate_id, metadata.topic, self.state())
    }
    fn message_clone(&self) -> Box<dyn Message>;

    fn state(&self) -> String;
}
impl_downcast!(Message);
impl Debug for dyn Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.metadata().topic)
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
