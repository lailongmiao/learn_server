use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use crate::models::User;

pub async fn get_users(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email
        FROM users
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch users: {}", e),
        )
    })?;

    Ok(Json(users))
} 
