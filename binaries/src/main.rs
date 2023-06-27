mod error;
mod routes;

use axum::{
    http::{HeaderValue, Method},
    Router,
};
use routes::board_routers;
use service_library::adapters::database::{AtomicConnection, Connection};
use service_library::domain::board::commands::*;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    println!("Environment Variable Is Being Set...");
    dotenv::dotenv().expect("Unable to load environment variable!");

    // ! OpenAPI
    #[derive(OpenApi)]
    #[openapi(
    paths(
        routes::create_board,
        routes::edit_board,
        routes::add_comment,
        routes::edit_comment
    ),
    components(
        schemas(
            CreateBoard,
            EditBoard,
            AddComment,
            EditComment)
    ),
    tags(
        (name= "Rustiful Backend", description="This is for swagger integration")
    )
    )]
    pub struct ApiDoc;

    // ! Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "tracing=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ! Connection
    println!("Connections Are Being Pooled...");
    let conn: AtomicConnection = Connection::new()
        .await
        .expect("Connection Creation Failed!");

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/boards", board_routers())
        .with_state(conn.clone())
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::PUT,
                    Method::DELETE,
                ]),
        )
        .layer(TraceLayer::new_for_http());

    println!("Binding...");
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
