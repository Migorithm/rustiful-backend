use uuid::Uuid;

use super::entity::{BoardState, CommentState};
use crate::{
    domain::{Message, MessageMetadata},
    message,
};
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

message!(BoardCreated, externally_notifiable, internally_notifiable);
message!(BoardUpdated);
message!(BoardCommentAdded);
