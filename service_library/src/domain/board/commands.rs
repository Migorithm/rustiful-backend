use serde::Deserialize;

use utoipa::ToSchema;
use uuid::Uuid;

use super::entity::BoardState;
use crate::domain::commands::Command;

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct CreateBoard {
    pub author: Uuid,
    pub title: String,
    pub content: String,
    pub state: BoardState,
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct EditBoard {
    pub id: Uuid,
    pub title: Option<String>,
    pub content: Option<String>,
    pub state: Option<BoardState>,
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct AddComment {
    pub board_id: Uuid,
    pub author: Uuid,
    pub content: String,
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct EditComment {
    pub board_id: Uuid,
    pub id: Uuid,
    pub content: String,
}

impl Command for CreateBoard {}
impl Command for EditBoard {}
impl Command for AddComment {}
impl Command for EditComment {}
