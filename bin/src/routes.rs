use std::sync::Arc;

use axum::routing::post;
use axum::Router;
use axum::{extract::State, Json};
use library::domain::commands::ServiceResponse;

use crate::error::{Exception, WebResponse};
use library::domain::board::commands::*;
use library::services::messagebus::MessageBus;

#[utoipa::path( post,  path = "/boards", request_body=CreateBoard)]
#[axum_macros::debug_handler]
pub async fn create_board(
    State(bus): State<Arc<MessageBus>>,
    Json(cmd): Json<CreateBoard>,
) -> Result<WebResponse<ServiceResponse>, Exception> {
    let res = bus.handle(cmd).await.map_err(Exception)?;

    Ok(WebResponse(res))
}

#[utoipa::path(patch, path = "/boards",request_body=EditBoard)]
#[axum_macros::debug_handler]
pub async fn edit_board(
    State(bus): State<Arc<MessageBus>>,
    Json(cmd): Json<EditBoard>,
) -> Result<WebResponse<ServiceResponse>, Exception> {
    let res = bus.handle(cmd).await.map_err(Exception)?;

    Ok(WebResponse(res))
}

#[utoipa::path(post, path = "/boards/comments",request_body=AddComment)]
pub async fn add_comment(
    State(bus): State<Arc<MessageBus>>,
    Json(cmd): Json<AddComment>,
) -> Result<WebResponse<ServiceResponse>, Exception> {
    let res = bus.handle(cmd).await.map_err(Exception)?;

    Ok(WebResponse(res))
}

#[utoipa::path(patch, path = "/boards/comments",request_body=EditComment)]
pub async fn edit_comment(
    State(bus): State<Arc<MessageBus>>,
    Json(cmd): Json<EditComment>,
) -> Result<WebResponse<ServiceResponse>, Exception> {
    let res = bus.handle(cmd).await.map_err(Exception)?;

    Ok(WebResponse(res))
}

pub fn board_routers() -> Router<Arc<MessageBus>> {
    Router::new()
        .route("/", post(create_board).patch(edit_board))
        .route("/comments", post(add_comment).patch(edit_comment))
}
