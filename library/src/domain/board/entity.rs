use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::State;
use serde::{Deserialize, Serialize};
use sqlx;

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash, sqlx::Type, Debug)]
#[sqlx(type_name = "board_state")]
pub enum BoardState {
    Unpublished,
    Published,
    Deleted,
}

impl Default for BoardState {
    fn default() -> Self {
        Self::Unpublished
    }
}
impl State for BoardState {
    fn state(&self) -> &str {
        match self {
            Self::Unpublished => "Unpublished",
            Self::Published => "Published",
            Self::Deleted => "Deleted",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Board {
    pub id: Uuid,
    pub author: Uuid,
    pub title: String,
    pub content: String,
    pub state: BoardState,
    pub create_dt: DateTime<Utc>,
    pub version: i32,
}

impl Board {
    pub fn new(
        author: Uuid,
        title: impl Into<String>,
        content: impl Into<String>,
        state: BoardState,
    ) -> Self {
        Self {
            author,
            title: title.into(),
            content: content.into(),
            state,
            id: Uuid::new_v4(),
            create_dt: Utc::now(),
            version: 0,
        }
    }
    pub fn state(&self) -> &str {
        self.state.state()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Comment {
    pub id: Uuid,
    pub board_id: Uuid,
    pub author: Uuid,
    pub content: String,
    pub state: CommentState,
    pub create_dt: DateTime<Utc>,
}

impl Comment {
    pub fn new(board_id: Uuid, author: Uuid, content: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            board_id,
            author,
            content: content.into(),
            state: CommentState::Pending,
            create_dt: Utc::now(),
        }
    }
    pub fn state(&self) -> &str {
        self.state.state()
    }
}

#[derive(
    Clone, PartialEq, PartialOrd, Eq, Ord, Debug, sqlx::Type, Default, Hash, Deserialize, Serialize,
)]
#[sqlx(type_name = "comment_state")]
pub enum CommentState {
    #[default]
    Created,
    Deleted,
    Pending,
    UpdatePending,
}

impl State for CommentState {
    fn state(&self) -> &str {
        match self {
            Self::Created => "Created",
            Self::Deleted => "Deleted",
            Self::Pending => "Pending",
            Self::UpdatePending => "UpdatePending",
        }
    }
}
