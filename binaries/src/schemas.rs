use std::any::Any;

use serde::Deserialize;
use service_library::domain::{board::entity::BoardState, commands::ApplicationCommand, AnyTrait};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateBoard {
    author: Uuid,
    title: String,
    content: String,
    state: BoardState,
}

#[derive(Debug, Deserialize)]
pub struct EditBoard {
    id: Uuid,
    title: Option<String>,
    content: Option<String>,
    state: Option<BoardState>,
}

#[derive(Debug, Deserialize)]
pub struct AddComment {
    board_id: Uuid,
    author: Uuid,
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct EditComment {
    board_id: Uuid,
    id: Uuid,
    content: String,
}

pub trait ToCommand: Send + Sync {
    fn to_command(self) -> Box<dyn Any + Sync + Send>;
}
impl ToCommand for CreateBoard {
    fn to_command(self) -> Box<dyn Any + Sync + Send> {
        ApplicationCommand::CreateBoard {
            author: self.author,
            title: self.title,
            content: self.content,
            state: self.state,
        }
        .as_any()
    }
}
impl ToCommand for EditBoard {
    fn to_command(self) -> Box<dyn Any + Sync + Send> {
        ApplicationCommand::EditBoard {
            id: self.id,
            title: self.title,
            content: self.content,
            state: self.state,
        }
        .as_any()
    }
}

impl ToCommand for AddComment {
    fn to_command(self) -> Box<dyn Any + Sync + Send> {
        ApplicationCommand::AddComment {
            board_id: self.board_id,
            author: self.author,
            content: self.content,
        }
        .as_any()
    }
}

impl ToCommand for EditComment {
    fn to_command(self) -> Box<dyn Any + Sync + Send> {
        ApplicationCommand::EditComment {
            board_id: self.board_id,
            id: self.id,
            content: self.content,
        }
        .as_any()
    }
}
