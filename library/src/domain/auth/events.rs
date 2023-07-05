use uuid::Uuid;

use super::entity::AccountState;
use crate::{
    domain::{Message, MessageMetadata},
    message,
};
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

message!(AccountCreated);
message!(AccountUpdated);
