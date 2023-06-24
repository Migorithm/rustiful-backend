mod error;
mod routes;
mod schemas;

use axum::Router;
use routes::board_routers;
use service_library::adapters::database::{AtomicConnection, Connection};

#[tokio::main]
async fn main() {
    println!("Environment Variable Is Being Set...");
    dotenv::dotenv().expect("Unable to load environment variable!");

    println!("Connections Are Being Pooled...");
    let conn: AtomicConnection = Connection::new()
        .await
        .expect("Connection Creation Failed!");

    let board_routes = board_routers();

    let app = Router::new()
        .nest("/boards", board_routes)
        .with_state(conn.clone());

    println!("Binding...");
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
