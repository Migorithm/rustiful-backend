use serde::Deserialize;
use service_library::domain::{board::entity::BoardState, commands::ApplicationCommand};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBoard {
    author: Uuid,
    title: String,
    content: String,
    state: BoardState,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EditBoard {
    id: Uuid,
    title: Option<String>,
    content: Option<String>,
    state: Option<BoardState>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddComment {
    board_id: Uuid,
    author: Uuid,
    content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EditComment {
    board_id: Uuid,
    id: Uuid,
    content: String,
}

pub trait ToCommand: Send + Sync {
    fn to_command(self) -> ApplicationCommand;
}
impl ToCommand for CreateBoard {
    fn to_command(self) -> ApplicationCommand {
        ApplicationCommand::CreateBoard {
            author: self.author,
            title: self.title,
            content: self.content,
            state: self.state,
        }
    }
}
impl ToCommand for EditBoard {
    fn to_command(self) -> ApplicationCommand {
        ApplicationCommand::EditBoard {
            id: self.id,
            title: self.title,
            content: self.content,
            state: self.state,
        }
    }
}

impl ToCommand for AddComment {
    fn to_command(self) -> ApplicationCommand {
        ApplicationCommand::AddComment {
            board_id: self.board_id,
            author: self.author,
            content: self.content,
        }
    }
}

impl ToCommand for EditComment {
    fn to_command(self) -> ApplicationCommand {
        ApplicationCommand::EditComment {
            board_id: self.board_id,
            id: self.id,
            content: self.content,
        }
    }
}
