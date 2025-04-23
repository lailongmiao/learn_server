use argon2::{
    Argon2, PasswordHasher, password_hash,
    password_hash::{PasswordHash, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router,
    extract::{Form, FromRequest, Path, Request, State, rejection},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use http::StatusCode;
use regex::Regex;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
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
    #[error(transparent)]
    JsonRejectionError(#[from] rejection::JsonRejection),
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ServerError::DatabaseError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::PasswordHashError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::JsonRejectionError(rejection) => {
                (rejection.status(), rejection.body_text())
            }
        };
        (status, Json(ErrorResponse { message })).into_response()
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
    #[validate(length(min = 3, max = 30,))]
    username: String,
    #[validate(email())]
    #[validate(length(max = 50))]
    email: String,
    #[validate(length(min = 6))]
    #[validate(regex(path = *UPPERCASE_RE))]
    #[validate(must_match(other = "confirm_password"))]
    password: String,
    #[validate(must_match(other = "password"))]
    confirm_password: String,
}

#[derive(Deserialize, Serialize, Validate)]
struct LoginInfo {
    #[validate(length(min = 3, max = 30,))]
    username: String,
    #[validate(length(min = 6))]
    #[validate(regex(path = *UPPERCASE_RE))]
    password: String,
}

struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ServerError;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(info) = Json::<T>::from_request(req, state).await?;
        info.validate()?;
        Ok(ValidatedJson(info))
    }
}

#[derive(Deserialize, Serialize)]
struct UserQuery {
    username: String,
    email: String,
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
        .route("/api/users/find", post(find_user_by_form))
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

async fn find_user_by_form(
    State(pool): State<PgPool>,
    Form(query): Form<UserQuery>,
) -> Result<Json<User>, ServerError> {
    let user = sqlx::query_as!(
        User,
        r#"
SELECT * 
FROM users 
WHERE username = $1 AND primary_email_address = $2
        "#,
        query.username,
        query.email
    )
    .fetch_one(&pool)
    .await?;
    Ok(Json(user))
}

async fn register_user(
    State(pool): State<PgPool>,
    ValidatedJson(register_info): ValidatedJson<RegisterInfo>,
) -> Result<(), ServerError> {
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
    ValidatedJson(login_info): ValidatedJson<LoginInfo>,
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
    ARGON2.verify_password(login_info.password.as_bytes(), &hash)?;
    Ok(Json(search_user))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };

    use http_body_util::BodyExt;
    use sqlx::postgres::PgPool;
    use tower::ServiceExt;

    async fn create_test_app() -> Router {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();
        Router::new()
            .route("/api/register", post(register_user))
            .route("/api/users/find", post(find_user_by_form))
            .with_state(pool)
    }

    async fn get_html(response: Response<Body>) -> String {
        let body = response.into_body();
        let collected = body.collect().await.unwrap();
        let bytes = collected.to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    async fn drop_data(info: RegisterInfo) {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap(); // 使用 .await 解包 Future
        let _ = sqlx::query_as!(
            User,
            r#"
DELETE FROM users 
WHERE username = $1
            "#,
            info.username,
        )
        .execute(&pool)
        .await
        .unwrap();
    }

    async fn create_data() {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();
        let _ = sqlx::query_as!(
            User,
            r#"
INSERT INTO users (username,primary_email_address,password,team_id,group_id)
VALUES ($1,$2,$3,$4,$5)
            "#,
            "haoxiangzhou",
            "haoxiangzhou@example.com",
            "P2025zhx",
            Option::<Uuid>::None,
            Option::<Uuid>::None
        )
        .execute(&pool)
        .await
        .unwrap();
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
        drop_data(register_info).await;
    }

    #[tokio::test]
    async fn test_register_user_failed() {
        let app = create_test_app().await;
        let register_info = RegisterInfo {
            username: "test_2".to_string(),
            email: "test_2@example.com".to_string(),
            password: "p2025test2".to_string(),
            confirm_password: "p2025test2".to_string(),
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
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let html = get_html(response).await;
        assert_eq!(
            html,
            "Validation Error: password: passwords must contain at least one upper case letter"
        );
    }

    #[tokio::test]
    async fn test_find_user_by_form_success() {
        create_data().await;
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/users/find")
                    .method("POST")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(Body::from("username=haoxiangzhou&email=haoxiangzhou@example.com"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        drop_data(RegisterInfo {
            username: "haoxiangzhou".to_string(),
            email: "haoxiangzhou@example.com".to_string(),
            password: "P2025zhx".to_string(),
            confirm_password: "P2025zhx".to_string(),
        })
        .await;
    }
}
