mod entity;
pub mod events;
use std::{collections::VecDeque, mem};

use crate::aggregate;

use self::entity::{Account, TokenStat};

use super::{
    builder::{Buildable, Builder},
    Aggregate, Message,
};

aggregate!(AuthAggregate);

#[derive(Default)]
pub struct AuthAggregate {
    pub account: Account,
    pub token_stat: TokenStat,
    pub events: VecDeque<Box<dyn Message>>, //Event
}

pub struct AuthAggregateBuilder(AuthAggregate);

impl AuthAggregateBuilder {
    pub fn take_account(mut self, account: Account) -> Self {
        self.0.account = account;
        self
    }
    pub fn take_token_stat(mut self, token_stat: TokenStat) -> Self {
        self.0.token_stat = token_stat;
        self
    }
}

impl Builder<AuthAggregate> for AuthAggregateBuilder {
    fn new() -> Self {
        Self(AuthAggregate::default())
    }

    fn build(self) -> AuthAggregate {
        self.0
    }
}

impl Buildable<AuthAggregate, AuthAggregateBuilder> for AuthAggregate {
    fn builder() -> AuthAggregateBuilder {
        AuthAggregateBuilder::new()
    }
}

#[test]
fn test_create_account() {
    let auth_builder = AuthAggregate::builder();
    let account = Account::new(
        "Migo".into(),
        "migo@mail.com".into(),
        "testpass".into(),
        "Mago".into(),
    );
    let auth_aggregate = auth_builder.take_account(account).build();

    assert_ne!(
        auth_aggregate.account.hashed_password,
        "testpass".to_string()
    );
    assert!(bcrypt::verify("testpass", &auth_aggregate.account.hashed_password).unwrap());
    assert!(auth_aggregate.account.verify_password("testpass"))
}
