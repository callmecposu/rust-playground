use std::fs;

use axum::{ routing::{ get, post }, Router, http::StatusCode, Json };
use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize, Clone)]
struct User {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    let users_string = fs::read_to_string("./src/users.json").unwrap();
    let users = serde_json::from_str::<Vec<User>>(users_string.as_str()).unwrap();

    let app = Router::new().route(
        "/",
        get(|| async { (StatusCode::OK, Json(users)) })
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
