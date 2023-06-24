mod schemas;
use schemas::ToCommand;
use serde_json::json;
use std::{any::Any, time::Duration};
use tower_http::trace::TraceLayer;

use axum::{
    error_handling::HandleErrorLayer,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use service_library::{
    adapters::database::{AtomicConnection, Connection},
    services::messagebus::MessageBus,
    utils::ApplicationError,
};
use tower::{BoxError, ServiceBuilder};
struct Exception(ApplicationError);

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

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Unable to load environment variable!");

    let conn: AtomicConnection = Connection::new()
        .await
        .expect("Connection Creation Failed!");
    let app = Router::new()
        .route("/boards", post(create_board))
        .with_state(conn.clone());
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[axum_macros::debug_handler]
async fn create_board(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<schemas::CreateBoard>,
) -> Result<String, Exception> {
    let cmd: Box<dyn Any + Send + Sync> = cmd.to_command();

    let mut bus = MessageBus::new(None);
    let mut res = bus.handle(cmd, connection).await.map_err(Exception)?;

    Ok(res.pop_front().unwrap())
}
