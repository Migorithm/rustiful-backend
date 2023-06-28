use uuid::Uuid;

use super::entity::AccountState;
use crate::domain::{Message, MessageClone, MessageMetadata};
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub enum AuthEvent {
    Created {
        id: Uuid,
        author: Uuid,
        title: String,
        content: String,
        state: AccountState,
    },
    Updated {
        id: Uuid,
        title: Option<String>,
        content: Option<String>,
        state: Option<AccountState>,
    },
}

pub const TOPIC: &str = "auth";

impl Message for AuthEvent {
    fn metadata(&self) -> MessageMetadata {
        match self {
            Self::Created { id, .. } | Self::Updated { id, .. } => MessageMetadata {
                aggregate_id: id.to_string(),
                topic: TOPIC.into(),
            },
        }
    }
    fn state(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }
    fn externally_notifiable(&self) -> bool {
        match self {
            Self::Created { .. } => false,
            Self::Updated { .. } => false,
        }
    }
}

impl MessageClone for AuthEvent {
    fn message_clone(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
}
