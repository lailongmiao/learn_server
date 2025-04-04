mod models;
mod db;
mod handlers;

use axum::{
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = db::create_pool()
        .await
        .expect("Failed to create pool");

    let app = Router::new()
        .route("/api/users", get(handlers::get_users))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Server running on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await.unwrap();
}
