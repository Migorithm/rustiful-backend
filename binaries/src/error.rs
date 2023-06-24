use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use service_library::utils::ApplicationError;

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
            ApplicationError::InExecutableEvent => {
                (StatusCode::INTERNAL_SERVER_ERROR, "".to_string())
            }
            ApplicationError::InvalidURL => (StatusCode::BAD_REQUEST, "".to_string()),
            ApplicationError::NotFound => (StatusCode::NOT_FOUND, "Object Not Found!".to_string()),
            ApplicationError::TransactionError => {
                (StatusCode::BAD_GATEWAY, "Transaction Failed!".to_string())
            }
            ApplicationError::StopSentinel => {
                (StatusCode::LOCKED, "Stop Sentinel Detected!".to_string())
            }
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
