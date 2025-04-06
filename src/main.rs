use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    email: String,
}

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:zhx2004101@localhost:5432/axum_demo")
        .await
        .unwrap();
        
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
        
    let app = Router::new()
        .route("/api/users", get(get_users))
        .with_state(pool)
        .layer(cors);
        
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_users(State(pool): State<PgPool>) -> Json<Vec<User>> {
    let users = sqlx::query_as!(User, "SELECT id, username, email FROM users")
        .fetch_all(&pool)
        .await
        .unwrap();
    Json(users)
}
