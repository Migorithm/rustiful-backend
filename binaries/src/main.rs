mod error;
mod routes;
mod schemas;

use axum::{
    http::{HeaderValue, Method},
    Router,
};
use routes::board_routers;
use service_library::adapters::database::{AtomicConnection, Connection};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    println!("Environment Variable Is Being Set...");
    dotenv::dotenv().expect("Unable to load environment variable!");

    println!("Connections Are Being Pooled...");
    let conn: AtomicConnection = Connection::new()
        .await
        .expect("Connection Creation Failed!");

    let board_routes = board_routers();
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
            schemas::CreateBoard,
            schemas::EditBoard,
            schemas::AddComment,
            schemas::EditComment)
    ),
    tags(
        (name= "Rustiful Backend", description="This is for swagger integration")
    )
    )]
    pub struct ApiDoc;

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/boards", board_routes)
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
        );

    println!("Binding...");
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
