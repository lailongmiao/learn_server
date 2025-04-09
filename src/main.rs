use axum::{extract::{State,Path}, routing::{get,post}, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use tower_http::cors::{Any, CorsLayer};
use axum::http::StatusCode;


#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    email: String,
    team_id: Option<i32>,
    group_id: Option<i32>,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Team{
    id: i32,
    name: String,
}

#[derive(Serialize,Deserialize)]
struct Group{
    id: i32,
    name: String,
    team_id:i32,
}

#[derive(Deserialize)]
struct RegisterInfo{
    username:String,
    email:String,
    password:String,
}

#[derive(Deserialize)]
struct LoginInfo{
    username:String,
    password:String,
}

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/axum_demo")
        .await
        .unwrap();
        
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
        
    let app = Router::new()
        .route("/api/users", get(get_users))
        .route("/api/teams",get(get_teams))
        .route("/api/teams/{team_id}/users",get(get_users_by_team_id_path))
        .route("/api/groups",get(get_groups))
        .route("/api/teams/{team_id}/groups",get(get_groups_by_team_id))
        .route("/api/groups/{group_id}/users",get(get_users_by_group_id))
        .route("/api/register",post(register_user))
        .route("/api/login",post(login_user))
        .with_state(pool)
        .layer(cors);
        
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_users(State(pool): State<PgPool>) -> Json<Vec<User>> {
    let users = sqlx::query_as!(User, "SELECT id, username, email,team_id,group_id,password FROM users")
    .fetch_all(&pool)
    .await
    .unwrap();
    Json(users)
}

async fn get_teams(State(pool):State<PgPool>) -> Json<Vec<Team>>
{
    let teams=sqlx::query_as!(Team,"SELECT id,name from teams")
    .fetch_all(&pool)
    .await
    .unwrap();
    Json(teams)
}

async fn get_users_by_team_id_path(State(pool):State<PgPool>,Path(team_id):Path<i32>) ->Json<Vec<User>>
{
    let users=sqlx::query_as!(User,"SELECT id,username,email,team_id,group_id ,password from users where team_id=$1",team_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    Json(users)
}

async fn get_groups(State(pool):State<PgPool>) ->Json<Vec<Group>>
{
    let groups=sqlx::query_as!(Group,"SELECT id,name,team_id from groups")
    .fetch_all(&pool)
    .await
    .unwrap();
    Json(groups)
}

async fn get_groups_by_team_id(State(pool):State<PgPool>,Path(team_id):Path<i32>) ->Json<Vec<Group>>
{
    let groups=sqlx::query_as!(Group,"SELECT id,name,team_id from groups where team_id=$1",team_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    Json(groups)
}

async fn get_users_by_group_id(State(pool): State<PgPool>,Path(group_id): Path<i32>) -> Json<Vec<User>> 
{    
    let users = sqlx::query_as!(User,"SELECT id, username, email, team_id, group_id ,password FROM users WHERE group_id = $1",group_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    
    Json(users)
}

async fn register_user(State(pool):State<PgPool>,Json(register_info):Json<RegisterInfo>)->Result<Json<User>,StatusCode>
{
    let new_user=sqlx::query_as!(User,"INSERT INTO users (username,email,password,team_id,group_id) VALUES ($1,$2,$3,$4,$5) RETURNING id,username,email,team_id,group_id,password",register_info.username,register_info.email,register_info.password,Option::<i32>::None,Option::<i32>::None)
    .fetch_one(&pool)
    .await
    .map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(new_user))
}

async fn login_user(State(pool):State<PgPool>,Json(login_info):Json<LoginInfo>)->Result<Json<User>,StatusCode>
{
    let search_user=sqlx::query_as!(User,"SELECT id,username,email,team_id,group_id,password FROM users WHERE username=$1 AND password=$2",login_info.username,login_info.password)
    .fetch_optional(&pool)
    .await
    .map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;

    match search_user
    {
        Some(user)=>Ok(Json(user)),
        None=>Err(StatusCode::UNAUTHORIZED),
    }

}