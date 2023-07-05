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
    fn internally_notifiable(&self) -> bool {
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

#[macro_export]
macro_rules! message {
    ($event:ty ) => {
        impl Message for $event {
            fn metadata(&self) -> MessageMetadata {
                MessageMetadata {
                    aggregate_id: self.id.to_string(),
                    topic: stringify!($event).into(),
                }
            }
            fn message_clone(&self) -> Box<dyn Message> {
                Box::new(self.clone())
            }
            fn state(&self) -> String {
                serde_json::to_string(&self).expect("Failed to serialize")
            }
        }
    };
    ($event:ty , $v1:ident) => {
        impl Message for $event {
            fn metadata(&self) -> MessageMetadata {
                MessageMetadata {
                    aggregate_id: self.id.to_string(),
                    topic: stringify!($event).into(),
                }
            }
            fn message_clone(&self) -> Box<dyn Message> {
                Box::new(self.clone())
            }
            fn state(&self) -> String {
                serde_json::to_string(&self).expect("Failed to serialize")
            }
            fn $v1ly_notifiable(&self) -> bool {
                true
            }
        }
    };
    ($event:ty , $v1:ident, $v2:ident) => {
        impl Message for $event {
            fn metadata(&self) -> MessageMetadata {
                MessageMetadata {
                    aggregate_id: self.id.to_string(),
                    topic: stringify!($event).into(),
                }
            }
            fn message_clone(&self) -> Box<dyn Message> {
                Box::new(self.clone())
            }
            fn state(&self) -> String {
                serde_json::to_string(&self).expect("Failed to serialize")
            }
            fn $v1(&self) -> bool {
                true
            }
            fn $v2(&self) -> bool {
                true
            }
        }
    };
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

#[macro_export]
macro_rules! aggregate {
    ($aggregate:ty) => {
        impl Aggregate for $aggregate {
            fn events(&self) -> &VecDeque<Box<dyn Message>> {
                &self.events
            }
            fn take_events(&mut self) -> VecDeque<Box<dyn Message>> {
                mem::take(&mut self.events)
            }
            fn raise_event(&mut self, event: Box<dyn Message>) {
                self.events.push_back(event)
            }
        }

        impl AsRef<$aggregate> for $aggregate {
            fn as_ref(&self) -> &$aggregate {
                self
            }
        }
        impl AsMut<$aggregate> for $aggregate {
            fn as_mut(&mut self) -> &mut $aggregate {
                self
            }
        }
    };
}
