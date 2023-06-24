use axum::routing::post;
use axum::Router;
use axum::{extract::State, Json};

use service_library::{adapters::database::AtomicConnection, services::messagebus::MessageBus};

use crate::error::{Exception, WebResponse};
use crate::schemas::{ToCommand, *};

#[utoipa::path( post,  path = "/boards", request_body=CreateBoard)]
pub async fn create_board(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<CreateBoard>,
) -> Result<WebResponse, Exception> {
    let mut bus = MessageBus::new(None);
    let mut res = bus
        .handle(cmd.to_command(), connection)
        .await
        .map_err(Exception)?;

    Ok(WebResponse(res.pop_front().unwrap()))
}

#[utoipa::path(patch, path = "/boards",request_body=EditBoard)]
pub async fn edit_board(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<EditBoard>,
) -> Result<WebResponse, Exception> {
    let mut bus = MessageBus::new(None);
    let mut res = bus
        .handle(cmd.to_command(), connection)
        .await
        .map_err(Exception)?;

    Ok(WebResponse(res.pop_front().unwrap()))
}

#[utoipa::path(post, path = "/boards/comments",request_body=AddComment)]
pub async fn add_comment(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<AddComment>,
) -> Result<WebResponse, Exception> {
    let mut bus = MessageBus::new(None);
    let mut res = bus
        .handle(cmd.to_command(), connection)
        .await
        .map_err(Exception)?;

    Ok(WebResponse(res.pop_front().unwrap()))
}

#[utoipa::path(patch, path = "/boards/comments",request_body=EditComment)]
pub async fn edit_comment(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<EditComment>,
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
