use uuid::Uuid;

use super::entity::AccountState;
use crate::domain::{Message, MessageMetadata};
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub struct AccountCreated {
    id: Uuid,
    author: Uuid,
    title: String,
    content: String,
    state: AccountState,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub struct AccountUpdated {
    id: Uuid,
    title: Option<String>,
    content: Option<String>,
    state: Option<AccountState>,
}

impl Message for AccountCreated {
    fn metadata(&self) -> MessageMetadata {
        MessageMetadata {
            aggregate_id: self.id.to_string(),
            topic: "AccountCreated".to_string(),
        }
    }

    fn externally_notifiable(&self) -> bool {
        false
    }

    fn message_clone(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
    fn state(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }
}

impl Message for AccountUpdated {
    fn metadata(&self) -> MessageMetadata {
        MessageMetadata {
            aggregate_id: self.id.to_string(),
            topic: "AccountUpdated".to_string(),
        }
    }

    fn externally_notifiable(&self) -> bool {
        false
    }

    fn message_clone(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
    fn state(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }
}
