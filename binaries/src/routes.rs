use axum::routing::post;
use axum::Router;
use axum::{extract::State, Json};

use crate::error::{Exception, WebResponse};
use service_library::domain::board::commands::*;
use service_library::{adapters::database::AtomicConnection, services::messagebus::MessageBus};

#[utoipa::path( post,  path = "/boards", request_body=CreateBoard)]
#[axum_macros::debug_handler]
pub async fn create_board(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<CreateBoard>,
) -> Result<WebResponse<String>, Exception> {
    let mut bus = MessageBus::<CreateBoard>::new();
    let res = bus.handle(cmd, connection).await.map_err(Exception)?;

    Ok(WebResponse(res))
}

#[utoipa::path(patch, path = "/boards",request_body=EditBoard)]
pub async fn edit_board(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<EditBoard>,
) -> Result<WebResponse<()>, Exception> {
    let mut bus = MessageBus::<EditBoard>::new();
    bus.handle(cmd, connection).await.map_err(Exception)?;

    Ok(WebResponse(()))
}

#[utoipa::path(post, path = "/boards/comments",request_body=AddComment)]
pub async fn add_comment(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<AddComment>,
) -> Result<WebResponse<()>, Exception> {
    let mut bus = MessageBus::<AddComment>::new();
    bus.handle(cmd, connection).await.map_err(Exception)?;

    Ok(WebResponse(()))
}

#[utoipa::path(patch, path = "/boards/comments",request_body=EditComment)]
pub async fn edit_comment(
    State(connection): State<AtomicConnection>,
    Json(cmd): Json<EditComment>,
) -> Result<WebResponse<()>, Exception> {
    let mut bus = MessageBus::<EditComment>::new();
    bus.handle(cmd, connection).await.map_err(Exception)?;

    Ok(WebResponse(()))
}

pub fn board_routers() -> Router<AtomicConnection> {
    Router::new()
        .route("/", post(create_board).patch(edit_board))
        .route("/comments", post(add_comment).patch(edit_comment))
}
