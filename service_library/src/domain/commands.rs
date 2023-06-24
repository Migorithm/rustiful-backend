use crate::domain::board::entity::BoardState;

use serde::{self, Deserialize, Serialize};
use uuid::Uuid;

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub enum ApplicationCommand {
    CreateBoard {
        author: Uuid,
        title: String,
        content: String,
        state: BoardState,
    },
    EditBoard {
        id: Uuid,
        title: Option<String>,
        content: Option<String>,
        state: Option<BoardState>,
    },

    AddComment {
        board_id: Uuid,
        author: Uuid,
        content: String,
    },
    EditComment {
        board_id: Uuid,
        id: Uuid,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub enum ServiceResponse {
    String(String),
}

impl ServiceResponse {
    pub fn to_str(&self) -> String {
        match self {
            Self::String(var) => var.to_owned(),
        }
    }
}

impl From<String> for ServiceResponse {
    fn from(value: String) -> Self {
        ServiceResponse::String(value)
    }
}
impl From<Uuid> for ServiceResponse {
    fn from(value: Uuid) -> Self {
        ServiceResponse::String(value.to_string())
    }
}

#[test]
fn test_serde() {
    use serde_json;
    use std::str::FromStr;
    let cmd = ApplicationCommand::CreateBoard {
        author: Uuid::from_str("3264af5a-564d-46aa-b69c-e9cf1508255e").unwrap(),
        title: "Whatever".to_string(),
        content: "AnyContent".to_string(),
        state: BoardState::Unpublished,
    };

    let jsonified = serde_json::to_string(&cmd).unwrap();

    assert_eq!(
        jsonified,
        r#"{"CreateBoard":{"author":"3264af5a-564d-46aa-b69c-e9cf1508255e","title":"Whatever","content":"AnyContent","state":"Unpublished"}}"#
    );

    let _cmd_to_test :ApplicationCommand = serde_json::from_str(r#"{"CreateBoard":{"author":"3264af5a-564d-46aa-b69c-e9cf1508255e","title":"Whatever","content":"AnyContent","state":"Unpublished"}}"#).unwrap();

    matches!(cmd, _cmd_to_test);
}
