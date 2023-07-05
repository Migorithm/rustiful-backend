use crate::utils::ApplicationError;

use serde::{self, Serialize};

use uuid::Uuid;

pub trait Command: 'static + Send {}

#[derive(Debug, Clone, Serialize)]
pub enum ServiceResponse {
    String(String),
    Bool(bool),
    Empty(()),
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
impl From<bool> for ServiceResponse {
    fn from(value: bool) -> Self {
        ServiceResponse::Bool(value)
    }
}
impl From<()> for ServiceResponse {
    fn from(_value: ()) -> Self {
        ServiceResponse::Empty(())
    }
}

impl TryFrom<ServiceResponse> for String {
    type Error = ApplicationError;

    fn try_from(value: ServiceResponse) -> Result<Self, Self::Error> {
        match value {
            ServiceResponse::String(val) => Ok(val),
            _ => Err(ApplicationError::ParsingError),
        }
    }
}

impl TryFrom<ServiceResponse> for () {
    type Error = ApplicationError;

    fn try_from(value: ServiceResponse) -> Result<Self, Self::Error> {
        match value {
            ServiceResponse::Empty(()) => Ok(()),
            _ => Err(ApplicationError::ParsingError),
        }
    }
}

impl TryFrom<ServiceResponse> for bool {
    type Error = ApplicationError;
    fn try_from(value: ServiceResponse) -> Result<Self, Self::Error> {
        match value {
            ServiceResponse::Bool(bool) => Ok(bool),
            _ => Err(ApplicationError::ParsingError),
        }
    }
}
