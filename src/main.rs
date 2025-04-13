use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordVerifier, SaltString},
    Argon2, PasswordHasher,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::sync::LazyLock;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};
use validator::Validate;

static ARGON2: LazyLock<Argon2> = LazyLock::new(|| Argon2::default());

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    email: String,
    team_id: Option<i32>,
    group_id: Option<i32>,
    password: String,
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("用户不存在")]
    UserNotFound,
    #[error("密码错误")]
    InvalidPassword,
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("注册失败: {0}")]
    RegistrationError(String),
    #[error("验证错误: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    #[error("密码哈希错误")]
    PasswordHashError,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let error_response = match self {
            ServerError::UserNotFound => "用户不存在".to_string(),
            ServerError::InvalidPassword => "密码错误".to_string(),
            ServerError::DatabaseError(err) => format!("数据库错误: {}", err),
            ServerError::RegistrationError(err) => format!("注册失败: {}", err),
            ServerError::ValidationError(err) => format!("验证错误: {}", err),
            ServerError::PasswordHashError => "密码哈希错误".to_string(),
        };
        error_response.into_response()
    }
}

#[derive(Serialize, Deserialize)]
struct Team {
    id: i32,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Group {
    id: i32,
    name: String,
    team_id: i32,
}

#[derive(Deserialize, Validate, Clone)]
struct RegisterInfo {
    #[validate(length(min = 1))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 6))]
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
    Path(team_id): Path<i32>,
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
    Path(team_id): Path<i32>,
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
    Path(group_id): Path<i32>,
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
    let password_hash = hash_password(register_info.password)?;

    let _ = sqlx::query!(
        r#"
INSERT INTO users (username,email,password,team_id,group_id)
VALUES ($1,$2,$3,$4,$5)
        "#,
        register_info.username,
        register_info.email,
        password_hash,
        Option::<i32>::None,
        Option::<i32>::None
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
            let is_valid = verify_password(login_info.password, user.password.clone());

            if is_valid {
                Ok(Json(user))
            } else {
                Err(ServerError::InvalidPassword)
            }
        }
        None => Err(ServerError::UserNotFound),
    }
}

fn hash_password(password: String) -> Result<String, ServerError> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = ARGON2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| ServerError::PasswordHashError)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: String, password_hash: String) -> bool {
    match PasswordHash::new(&password_hash) {
        Ok(password_hash) => match ARGON2.verify_password(password.as_bytes(), &password_hash) {
            Ok(_) => true,
            Err(_) => false,
        },
        Err(_) => false,
    }
}
