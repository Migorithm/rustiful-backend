use uuid::Uuid;

use super::entity::AccountState;
use crate::domain::{Message, MessageMetadata};
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
    fn get_metadata(&self) -> MessageMetadata {
        match self {
            Self::Created { id, .. } | Self::Updated { id, .. } => MessageMetadata {
                aggregate_id: id.to_string(),
                topic: TOPIC.into(),
            },
        }
    }
    fn externally_notifiable(&self) -> bool {
        match self {
            Self::Created { .. } => false,
            Self::Updated { .. } => false,
        }
    }
}
