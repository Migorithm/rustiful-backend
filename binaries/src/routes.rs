use axum::routing::post;
use axum::Router;
use axum::{extract::State, Json};

use service_library::{adapters::database::AtomicConnection, services::messagebus::MessageBus};

use crate::error::{Exception, WebResponse};
use crate::schemas::{self, ToCommand};

#[axum_macros::debug_handler]
pub async fn create_board(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<schemas::CreateBoard>,
) -> Result<WebResponse, Exception> {
    let mut bus = MessageBus::new(None);
    let mut res = bus
        .handle(cmd.to_command(), connection)
        .await
        .map_err(Exception)?;

    Ok(WebResponse(res.pop_front().unwrap()))
}

#[axum_macros::debug_handler]
pub async fn edit_board(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<schemas::EditBoard>,
) -> Result<WebResponse, Exception> {
    let mut bus = MessageBus::new(None);
    let mut res = bus
        .handle(cmd.to_command(), connection)
        .await
        .map_err(Exception)?;

    Ok(WebResponse(res.pop_front().unwrap()))
}
#[axum_macros::debug_handler]
pub async fn add_comment(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<schemas::AddComment>,
) -> Result<WebResponse, Exception> {
    let mut bus = MessageBus::new(None);
    let mut res = bus
        .handle(cmd.to_command(), connection)
        .await
        .map_err(Exception)?;

    Ok(WebResponse(res.pop_front().unwrap()))
}

#[axum_macros::debug_handler]
pub async fn edit_comment(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<schemas::EditComment>,
) -> Result<WebResponse, Exception> {
    let mut bus = MessageBus::new(None);
    let mut res = bus
        .handle(cmd.to_command(), connection)
        .await
        .map_err(Exception)?;

    Ok(WebResponse(res.pop_front().unwrap()))
}

pub fn board_routers() -> Router<AtomicConnection> {
    Router::new()
        .route("/", post(create_board).patch(edit_board))
        .route("/comments", post(add_comment).patch(edit_comment))
}
