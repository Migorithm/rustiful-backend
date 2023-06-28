use uuid::Uuid;

use super::entity::{BoardState, CommentState};
use crate::domain::{Message, MessageClone, MessageMetadata};
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub enum BoardEvent {
    Created {
        id: Uuid,
        author: Uuid,
        title: String,
        content: String,
        state: BoardState,
    },
    Updated {
        id: Uuid,
        title: Option<String>,
        content: Option<String>,
        state: Option<BoardState>,
    },
    CommentAdded {
        id: Uuid,
        author: Uuid,
        content: String,
        state: CommentState,
    },
}
pub const TOPIC: &str = "board";

impl Message for BoardEvent {
    fn metadata(&self) -> MessageMetadata {
        match self {
            Self::Created { id, .. } | Self::Updated { id, .. } | Self::CommentAdded { id, .. } => {
                MessageMetadata {
                    aggregate_id: id.to_string(),
                    topic: TOPIC.into(),
                }
            }
        }
    }
    fn state(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }

    fn externally_notifiable(&self) -> bool {
        match self {
            Self::Created { .. } => true,
            Self::Updated { .. } => false,
            Self::CommentAdded { .. } => false,
        }
    }
}
impl MessageClone for BoardEvent {
    fn message_clone(&self) -> Box<dyn Message> {
        Box::new(self.clone())
    }
}
