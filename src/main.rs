use argon2::{Argon2, PasswordHasher, password_hash::{PasswordHash, PasswordVerifier, SaltString, rand_core::OsRng}, password_hash};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use http::StatusCode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::sync::LazyLock;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;
use validator::Validate;

static ARGON2: LazyLock<Argon2> = LazyLock::new(Argon2::default);
static UPPERCASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r".*[A-Z].*").expect("Invalid uppercase regex pattern"));
#[derive(Serialize, Deserialize)]
struct User {
    id: Uuid,
    username: String,
    primary_email_address: String,
    organization_id: Option<Uuid>,
    team_id: Option<Uuid>,
    group_id: Option<Uuid>,
    password: String,
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error(transparent)]
    PasswordHashError(#[from] password_hash::Error),
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let error_response = match self {
            ServerError::DatabaseError(err) => (
                StatusCode::BAD_REQUEST,
                format!("Database Error: {}", err),
            ),
            ServerError::ValidationError(err) => (
                StatusCode::BAD_REQUEST,
                format!("Validation Error: {}", err),
            ),
            ServerError::PasswordHashError(err)=> (
                StatusCode::BAD_REQUEST,
                format!("Password_hash Error: {}", err),
            ),
        };
        error_response.into_response()
    }
}

#[derive(Serialize, Deserialize)]
struct Team {
    id: Uuid,
    name: String,
    organization_id: Uuid,
}

#[derive(Serialize, Deserialize)]
struct Group {
    id: Uuid,
    name: String,
    team_id: Uuid,
}

#[derive(Deserialize, Serialize, Validate)]
struct RegisterInfo {
    #[validate(length(
        min = 3,
        max = 30,
        message = "Username must be between 3 and 30 characters long"
    ))]
    username: String,
    #[validate(email(message = "Please enter a valid email address"))]
    #[validate(length(max = 50, message = "Email address length cannot exceed 50 characters"))]
    email: String,
    #[validate(length(min = 6, message = "The password must be at least 6 characters"))]
    #[validate(regex(path=*UPPERCASE_RE,message = "passwords must contain at least one upper case letter"))]
    #[validate(must_match(other = "confirm_password",message = "The two passwords you entered do not match."))]
    password: String,
    #[validate(must_match(other = "password",message = "The two passwords you entered do not match."))]
    confirm_password:String,
}

#[derive(Deserialize, Serialize, Validate)]
struct LoginInfo {
    #[validate(length(
        min = 3,
        max = 30,
        message = "The Username does not conform to validation rules"
    ))]
    username: String,
    #[validate(length(min = 6, message = "The password does not meet the verification rules"))]
    #[validate(regex(path=*UPPERCASE_RE,message = "passwords must contain at least one upper case letter"))]
    password: String,
}

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url).await.unwrap();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/users", get(get_users))
        .route("/api/teams", get(get_teams))
        .route("/api/teams/{team_id}/users", get(get_users_by_team_id_path))
        .route("/api/groups", get(get_groups))
        .route("/api/teams/{team_id}/groups", get(get_groups_by_team_id))
        .route("/api/groups/{group_id}/users", get(get_users_by_group_id))
        .route("/api/register", post(register_user))
        .route("/api/login", post(login_user))
        .with_state(pool)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_users(State(pool): State<PgPool>) -> Result<Json<Vec<User>>, ServerError> {
    let users = sqlx::query_as!(
        User,
        r#"
SELECT * 
FROM users 
ORDER BY id
        "#
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(users))
}

async fn get_teams(State(pool): State<PgPool>) -> Result<Json<Vec<Team>>, ServerError> {
    let teams = sqlx::query_as!(
        Team,
        r#"
SELECT * 
FROM teams 
ORDER BY id
        "#
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(teams))
}

async fn get_users_by_team_id_path(
    State(pool): State<PgPool>,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<User>>, ServerError> {
    let users = sqlx::query_as!(
        User,
        r#"
SELECT * 
FROM users 
WHERE team_id = $1
        "#,
        team_id
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(users))
}

async fn get_groups(State(pool): State<PgPool>) -> Result<Json<Vec<Group>>, ServerError> {
    let groups = sqlx::query_as!(
        Group,
        r#"
SELECT * 
FROM groups 
ORDER BY id
        "#
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(groups))
}

async fn get_groups_by_team_id(
    State(pool): State<PgPool>,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<Group>>, ServerError> {
    let groups = sqlx::query_as!(
        Group,
        r#"
SELECT * 
FROM groups 
WHERE team_id = $1
        "#,
        team_id
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(groups))
}

async fn get_users_by_group_id(
    State(pool): State<PgPool>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<User>>, ServerError> {
    let users = sqlx::query_as!(
        User,
        r#"
SELECT * 
FROM users
WHERE group_id = $1
        "#,
        group_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(users))
}

async fn register_user(
    State(pool): State<PgPool>,
    Json(register_info): Json<RegisterInfo>,
) -> Result<(), ServerError> {
    register_info.validate()?;
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = ARGON2
        .hash_password(register_info.password.as_bytes(), &salt)?
        .to_string();

    let _ = sqlx::query!(
        r#"
INSERT INTO users (username,primary_email_address,password,team_id,group_id)
VALUES ($1,$2,$3,$4,$5)
        "#,
        register_info.username,
        register_info.email,
        password_hash,
        Option::<Uuid>::None,
        Option::<Uuid>::None
    )
    .execute(&pool)
    .await?;
    Ok(())
}

async fn login_user(
    State(pool): State<PgPool>,
    Json(login_info): Json<LoginInfo>,
) -> Result<Json<User>, ServerError> {
    login_info.validate()?;
    let search_user = sqlx::query_as!(
        User,
        r#"
SELECT * 
FROM users 
WHERE username=$1
        "#,
        login_info.username,
    )
        .fetch_one(&pool)
        .await?;
    let hash = PasswordHash::new(&search_user.password)?;
    ARGON2.verify_password(login_info.password.as_bytes(),&hash)?;
    Ok(Json(search_user))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };

    use sqlx::postgres::PgPool;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn create_test_app() -> Router {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();
        Router::new()
            .route("/api/register", post(register_user))
            .with_state(pool)
    }

    async fn get_html(response: Response<Body>) -> String {
        let body = response.into_body();
        let collected = body.collect().await.unwrap();
        let bytes = collected.to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }


    #[tokio::test]
    async fn test_register_user_success() {
        let app = create_test_app().await;
        let register_info = RegisterInfo {
            username: "haoxiangzhou".to_string(),
            email: "haoxiangzhou@example.com".to_string(),
            password: "P2025zhx".to_string(),
            confirm_password: "P2025zhx".to_string(),
        };
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/register")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&register_info).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_register_user_failed(){
        let app =create_test_app().await;
        let register_info = RegisterInfo {
            username: "test_2".to_string(),
            email: "test_2@example.com".to_string(),
            password: "p2025test2".to_string(),
            confirm_password: "p2025test2".to_string(),
        };
        let response =app
            .oneshot(
                Request::builder()
                    .uri("/api/register")
                    .method("POST")
                    .header("Content-Type","application/json")
                    .body(Body::from(serde_json::to_string(&register_info).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
            assert_eq!(response.status(),StatusCode::BAD_REQUEST);
        let html = get_html(response).await;
        assert_eq!(html, "Validation Error: password: passwords must contain at least one upper case letter");

    }
}
