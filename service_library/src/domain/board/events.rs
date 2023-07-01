use uuid::Uuid;

use super::entity::{BoardState, CommentState};
use crate::domain::{Message, MessageMetadata};
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub struct BoardCreated {
    pub(crate) id: Uuid,
    pub(crate) author: Uuid,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) state: BoardState,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub struct BoardUpdated {
    pub(crate) id: Uuid,
    pub(crate) title: Option<String>,
    pub(crate) content: Option<String>,
    pub(crate) state: Option<BoardState>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub struct BoardCommentAdded {
    pub(crate) id: Uuid,
    pub(crate) author: Uuid,
    pub(crate) content: String,
    pub(crate) state: CommentState,
}

impl Message for BoardCreated {
    fn metadata(&self) -> MessageMetadata {
        MessageMetadata {
            aggregate_id: self.id.to_string(),
            topic: "BoardCreated".into(),
        }
    }
    fn externally_notifiable(&self) -> bool {
        true
    }

    fn message_clone(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
    fn state(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }
}

impl Message for BoardUpdated {
    fn metadata(&self) -> MessageMetadata {
        MessageMetadata {
            aggregate_id: self.id.to_string(),
            topic: "BoardUpdated".into(),
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

impl Message for BoardCommentAdded {
    fn metadata(&self) -> MessageMetadata {
        MessageMetadata {
            aggregate_id: self.id.to_string(),
            topic: "BoardCommentAdded".into(),
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
