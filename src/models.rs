use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
} 