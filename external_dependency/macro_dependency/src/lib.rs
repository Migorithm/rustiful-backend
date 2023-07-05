use ::std::any::Any;
use ::std::fmt::Debug;
use chrono::{DateTime, Utc};

use uuid::Uuid;
pub trait Message: Sync + Send + Any {
    fn external(&self) -> bool {
        false
    }
    fn internal(&self) -> bool {
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

#[derive(Debug, Clone)]
pub struct Outbox {
    pub id: Uuid,
    pub aggregate_id: String,
    pub topic: String,
    pub state: String,
    pub processed: bool,
    pub create_dt: DateTime<Utc>,
}

impl Outbox {
    pub fn new(aggregate_id: String, topic: String, state: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            topic,
            state,
            processed: false,
            create_dt: Default::default(),
        }
    }
}

pub struct MessageMetadata {
    pub aggregate_id: String,
    pub topic: String,
}

impl Debug for dyn Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.metadata().topic)
    }
}
