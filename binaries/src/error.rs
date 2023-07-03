use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use service_library::{domain::commands::ServiceResponse, utils::ApplicationError};

pub struct Exception(pub ApplicationError);

impl From<ApplicationError> for Exception {
    fn from(value: ApplicationError) -> Self {
        Exception(value)
    }
}

impl IntoResponse for Exception {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self.0 {
            ApplicationError::DatabaseConnectionError(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            ApplicationError::DeserializationError(err) => {
                (StatusCode::BAD_REQUEST, err.to_string())
            }
            ApplicationError::InExecutableEvent => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApplicationError::InExecutableEvent.to_string(),
            ),
            ApplicationError::InvalidURL => (
                StatusCode::BAD_REQUEST,
                ApplicationError::InvalidURL.to_string(),
            ),
            ApplicationError::NotFound => (
                StatusCode::NOT_FOUND,
                ApplicationError::NotFound.to_string(),
            ),
            ApplicationError::EventNotFound => (
                StatusCode::NOT_FOUND,
                ApplicationError::EventNotFound.to_string(),
            ),
            ApplicationError::TransactionError => (
                StatusCode::BAD_GATEWAY,
                ApplicationError::TransactionError.to_string(),
            ),
            ApplicationError::StopSentinel => (
                StatusCode::LOCKED,
                ApplicationError::StopSentinel.to_string(),
            ),
            ApplicationError::ParsingError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApplicationError::ParsingError.to_string(),
            ),
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

pub struct WebResponse<T>(pub T);

impl IntoResponse for WebResponse<ServiceResponse> {
    fn into_response(self) -> axum::response::Response {
        match self.0 {
            ServiceResponse::Bool(b) => {
                if b {
                    "true".into_response()
                } else {
                    "false".into_response()
                }
            }
            ServiceResponse::String(s) => s.into_response(),
            ServiceResponse::Empty(_) => ().into_response(),
        }
    }
}
