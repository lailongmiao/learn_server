use argon2::{
    Argon2, PasswordHasher,
    password_hash::{PasswordHash, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::sync::LazyLock;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;
use validator::{Validate, ValidationError};

static ARGON2: LazyLock<Argon2> = LazyLock::new(Argon2::default);
static LOWERCASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r".*[a-z].*").expect("Invalid lowercase regex pattern"));
static UPPERCASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r".*[A-Z].*").expect("Invalid uppercase regex pattern"));

static DIGIT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r".*\d.*").expect("Invalid uppercase regex pattern"));

#[derive(Serialize, Deserialize)]
struct User {
    id: Uuid,
    username: String,
    primary_email_address: String,
    team_id: Option<Uuid>,
    group_id: Option<Uuid>,
    password: String,
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("User does not exist")]
    UserNotFound,
    #[error("Password Error")]
    InvalidPassword,
    #[error("Database Error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error(":Registration Error{0}")]
    RegistrationError(String),
    #[error("Validation Error : {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    #[error("Password Hash Error")]
    PasswordHashError,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let error_response = match self {
            ServerError::UserNotFound => "User does not exist".to_string(),
            ServerError::InvalidPassword => "Password Error".to_string(),
            ServerError::DatabaseError(err) => format!("Database Error: {}", err),
            ServerError::RegistrationError(err) => format!("Registration Error: {}", err),
            ServerError::ValidationError(err) => format!("Validation Error: {}", err),
            ServerError::PasswordHashError => "Password Hash Error".to_string(),
        };
        error_response.into_response()
    }
}

#[derive(Serialize, Deserialize)]
struct Team {
    id: Uuid,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Group {
    id: Uuid,
    name: String,
    team_id: Uuid,
}

#[derive(Deserialize, Validate, Clone)]
struct RegisterInfo {
    #[validate(length(min = 1, max = 50))]
    username: String,
    #[validate(email)]
    #[validate(length(max = 100))]
    email: String,
    #[validate(length(min = 6))]
    #[validate(custom(function = "validate_password"))]
    password: String,
}

#[derive(Deserialize)]
struct LoginInfo {
    username: String,
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
        .hash_password(register_info.password.as_bytes(), &salt)
        .map_err(|_| ServerError::PasswordHashError)?
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
    let search_user = sqlx::query_as!(
        User,
        r#"
SELECT * 
FROM users 
WHERE username=$1
        "#,
        login_info.username,
    )
    .fetch_optional(&pool)
    .await?;

    match search_user {
        Some(user) => {
            let password_hash = match PasswordHash::new(&user.password) {
                Ok(hash) => hash,
                Err(_) => return Err(ServerError::PasswordHashError),
            };

            let is_valid = ARGON2
                .verify_password(login_info.password.as_bytes(), &password_hash)
                .is_ok();

            if is_valid {
                Ok(Json(user))
            } else {
                Err(ServerError::InvalidPassword)
            }
        }
        None => Err(ServerError::UserNotFound),
    }
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    let lowercase = LOWERCASE_RE.is_match(password);
    let uppercase = UPPERCASE_RE.is_match(password);
    let digit = DIGIT_RE.is_match(password);

    if lowercase && uppercase && digit {
        Ok(())
    } else {
        let error = ValidationError::new("password_error");
        Err(error)
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use sqlx::postgres::PgPool;
    #[tokio::test]
    async fn test_register_user()
    {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = PgPool::connect(&database_url).await.unwrap();
        let register_info =RegisterInfo{
            username: "test_user".to_string(),
            email:"testuser@example.com".to_string(),
            password:"Password123!".to_string(),
        };
        let result =register_user(State(pool),Json(register_info)).await;
        match result{
            Ok(_)=>println!("User registered successfully"),
            Err(ref e)=>println!("Failed to register user: {}",e),
        }
        assert!(result.is_ok());
    }
}