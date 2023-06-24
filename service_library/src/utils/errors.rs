use std::{error, fmt::Display};

pub type AnyError = dyn error::Error + Send + Sync + 'static;

#[derive(Debug)]
pub enum ApplicationError {
    AggregateConflict,
    DatabaseConnectionError(Box<AnyError>),
    DeserializationError(Box<AnyError>),
    UnexpectedError(Box<AnyError>),
    InExecutableCommand,
    InExecutableEvent,
    NotFound,
    InvalidURL,
    TransactionError,
    StopSentinel,
}

impl error::Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::AggregateConflict => write!(f, "AggregateConflict"),
            ApplicationError::DatabaseConnectionError(res) => write!(f, "{}", res),
            ApplicationError::DeserializationError(res) => write!(f, "{}", res),
            ApplicationError::UnexpectedError(res) => write!(f, "{}", res),
            ApplicationError::InExecutableCommand => write!(f, "InExecutableCommand"),
            ApplicationError::InExecutableEvent => write!(f, "InExecutableEvent"),
            ApplicationError::NotFound => write!(f, "NotFound"),
            ApplicationError::InvalidURL => write!(f, "InvalidURL"),
            ApplicationError::TransactionError => write!(f, "TransactionError"),
            ApplicationError::StopSentinel => write!(f, "StopSentinel"),
        }
    }
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;
