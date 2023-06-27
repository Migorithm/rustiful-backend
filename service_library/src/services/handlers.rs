use std::{pin::Pin, sync::Arc};

use crate::domain::auth::events::AuthEvent;
use crate::domain::board::events::BoardEvent;

use crate::utils::ApplicationResult;

use tokio::sync::Mutex;

use super::unit_of_work::UnitOfWork;

pub type Future<T> = Pin<Box<dyn futures::Future<Output = ApplicationResult<T>> + Send>>;

pub type CommandHandler<Command, Response> =
    Box<dyn Fn(Command, Arc<Mutex<UnitOfWork>>) -> Future<Response> + Send>;
pub type EventHandler<T> = fn(T, Arc<Mutex<UnitOfWork>>) -> Future<()>;

pub(crate) static BOARD_CREATED_EVENT_HANDLERS: [EventHandler<BoardEvent>; 0] = [];
pub(crate) static BOARD_UPDATED_EVENT_HANDLERS: [EventHandler<BoardEvent>; 0] = [];
pub(crate) static COMMENT_ADDED_EVENT_HANDLERS: [EventHandler<BoardEvent>; 0] = [];

pub(crate) static ACCOUNT_CREATED_EVENT_HANDLERS: [EventHandler<AuthEvent>; 0] = [];
pub(crate) static ACCOUNT_UPDATED_EVENT_HANDLERS: [EventHandler<AuthEvent>; 0] = [];
