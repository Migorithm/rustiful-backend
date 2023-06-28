use std::sync::Arc;

use crate::services::{handlers::Future, unit_of_work::UnitOfWork};

use serde::{self, Serialize};

use tokio::sync::Mutex;
use uuid::Uuid;

pub trait Command: 'static + Send {
    type Response;
    fn handle(self, uow: Arc<Mutex<UnitOfWork>>) -> Future<Self::Response>;
}

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

impl From<ServiceResponse> for String {
    fn from(value: ServiceResponse) -> Self {
        let ServiceResponse::String(var) = value else{
            panic!("Not possible")
        }  ;
        var
    }
}
