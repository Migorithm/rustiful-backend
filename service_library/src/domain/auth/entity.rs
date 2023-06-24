use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::State;

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash, sqlx::Type, Debug)]
#[sqlx(type_name = "board_state")]
pub enum AccountState {
    VerificationRequired,
    Created,
    Deleted,
    Blocked,
}

impl Default for AccountState {
    fn default() -> Self {
        Self::VerificationRequired
    }
}
impl State for AccountState {
    fn state(&self) -> &str {
        match self {
            Self::VerificationRequired => "VerificationRequired",
            Self::Created => "Created",
            Self::Deleted => "Deleted",
            Self::Blocked => "Blocked",
        }
    }
}

#[derive(Default)]
pub struct Account {
    //root
    id: String,
    email: String,
    state: AccountState,

    pub(super) hashed_password: String,
    nickname: String,
    create_dt: DateTime<Utc>,
    version: i32,
}

impl Account {
    pub fn new(id: String, email: String, password: String, nickname: String) -> Self {
        Self {
            id,
            email,
            state: Default::default(),
            hashed_password: Self::hash_password(password),
            nickname,
            create_dt: Default::default(),
            version: 0,
        }
    }
    pub fn hash_password(plain_password: String) -> String {
        bcrypt::hash(plain_password, 4).unwrap()
    }
    pub fn verify_password(&self, plain_password: impl AsRef<str>) -> bool {
        bcrypt::verify(plain_password.as_ref(), &self.hashed_password).is_ok()
    }
}
#[derive(Default)]
pub struct TokenStat {
    access_token: String,
    refresh_token: String,
    expiry_date: DateTime<Utc>,
}

impl TokenStat {
    pub fn create_token(&mut self) {
        todo!()
    }
}

#[test]
fn test_create_hashed_password() {
    let hashed_password = bcrypt::hash("whatever", 4).unwrap();
    assert_ne!(hashed_password, "whatever".to_string());
}

#[test]
fn test_verify_password() {
    let hashed_password = bcrypt::hash("whatever", 4).unwrap();
    assert_ne!(hashed_password, "whatever".to_string());
    assert!(bcrypt::verify("whatever", &hashed_password).unwrap());
}
