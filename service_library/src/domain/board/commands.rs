use serde::Deserialize;

use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::commands::Command;

use super::entity::BoardState;

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct CreateBoard {
    pub author: Uuid,
    pub title: String,
    pub content: String,
    pub state: BoardState,
}

impl Command for CreateBoard {}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct EditBoard {
    pub id: Uuid,
    pub title: Option<String>,
    pub content: Option<String>,
    pub state: Option<BoardState>,
}
impl Command for EditBoard {}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct AddComment {
    pub board_id: Uuid,
    pub author: Uuid,
    pub content: String,
}

impl Command for AddComment {}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct EditComment {
    pub board_id: Uuid,
    pub id: Uuid,
    pub content: String,
}

impl Command for EditComment {}
