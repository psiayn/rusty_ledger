use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use crate::state;
use time::OffsetDateTime;

use super::account::Types;

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    id: String,
    full_name: String,
    email: String,
    created_at: Option<OffsetDateTime>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserReq {
    full_name: String,
    email: String,
    account_type: Types,
}

#[axum::debug_handler]
pub async fn register(State(state): State<state::AppState>, Json(req): Json<UserReq>) {
    let pool = state.db;

    let user_id = sqlx::query_scalar!(
        r#"INSERT INTO users (full_name, email) VALUES ($1, $2) returning id"#,
        req.full_name,
        req.email
    ).fetch_one(&pool).await.unwrap();
    println!("user_id = {user_id}");
    let account_id = sqlx::query_scalar!(
        "INSERT INTO accounts (user_id, account_type) VALUES ($1, $2) returning id",
        user_id,
        req.account_type.to_string()
    ).fetch_one(&pool).await.unwrap();
    println!("account_id = {account_id}");
    let balance = sqlx::query_scalar!(
        "INSERT INTO account_balances (account_id) VALUES ($1) returning balance",
        account_id
    ).fetch_one(&pool).await.unwrap();
    println!("balance = {balance}");
}

#[axum::debug_handler]
pub async fn login() {
}

pub async fn update_profile() {
}

pub async fn delete() {
}
