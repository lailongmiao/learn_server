use axum::{extract::{State,Path}, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    email: String,
    team_id: Option<i32>,
    group_id: Option<i32>,
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
        .route("/api/teams",get(get_teams))
        .route("/api/teams/{team_id}/users",get(get_users_by_team_id_path))
        .route("/api/groups",get(get_groups))
        .route("/api/teams/{team_id}/groups",get(get_groups_by_team_id))
        .route("/api/teams/{team_id}/groups/{group_id}/users",get(get_users_by_team_group))
        .with_state(pool)
        .layer(cors);
        
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_users(State(pool): State<PgPool>) -> Json<Vec<User>> {
    let users = sqlx::query_as!(User, "SELECT id, username, email,team_id,group_id FROM users")
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
    let users=sqlx::query_as!(User,"SELECT id,username,email,team_id,group_id from users where team_id=$1",team_id)
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
    .await.unwrap();
    Json(groups)
}

async fn get_users_by_team_group(State(pool): State<PgPool>,Path((team_id, group_id)): Path<(i32, i32)>) -> Json<Vec<User>> 
{
    let is_group = sqlx::query!("SELECT id FROM groups WHERE id = $1 AND team_id = $2", group_id, team_id)
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_some();
    
    if !is_group {
        return Json(Vec::new());
    }
    
    let users = sqlx::query_as!(User,"SELECT id, username, email, team_id, group_id FROM users WHERE group_id = $1",group_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    
    Json(users)
}
