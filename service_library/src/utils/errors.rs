use std::{error, fmt::Display};

pub type AnyError = dyn error::Error + Send + Sync;

#[derive(Debug)]
pub enum ApplicationError {
    DatabaseConnectionError(Box<AnyError>),
    DeserializationError(Box<AnyError>),
    EntityNotFound,
    EventNotFound,
    CommandNotFound,
    InvalidURL,
    TransactionError,
    ParsingError,
    StopSentinel,
}

impl error::Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::DatabaseConnectionError(res) => write!(f, "{}", res),
            ApplicationError::DeserializationError(res) => write!(f, "{}", res),
            ApplicationError::EntityNotFound => write!(f, "EntityNotFound"),
            ApplicationError::CommandNotFound => write!(f, "CommandNotFound"),
            ApplicationError::EventNotFound => write!(f, "EventNotFound"),
            ApplicationError::InvalidURL => write!(f, "InvalidURL"),
            ApplicationError::TransactionError => write!(f, "TransactionError"),
            ApplicationError::StopSentinel => write!(f, "StopSentinel"),
            ApplicationError::ParsingError => write!(f, "ParsingError"),
        }
    }
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;
