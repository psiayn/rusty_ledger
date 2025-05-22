use axum::{extract::State, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use crate::state;
use jsonwebtoken::{encode, Header, EncodingKey};
use bcrypt::{hash, verify, DEFAULT_COST};

use super::account::Types;

#[derive(Clone, Serialize, Deserialize)]
struct User {
    id: Uuid,
    full_name: String,
    email: String,
    created_at: Option<OffsetDateTime>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserReq {
    full_name: String,
    email: String,
    password: String,
    account_type: Types,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateProfileReq {
    full_name: Option<String>,
    email: Option<String>,
    password: Option<String>,
    account_type: Option<Types>
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserRes {
    account_id: Uuid,
    full_name: String,
    email: String,
    account_type: Types,
    token: String
}

#[derive(Serialize, Deserialize)]
pub struct LoginReq {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRes {
    token: String,
    account_id: String,
    full_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,  // user id
    exp: usize,   // expiration time
}

#[axum::debug_handler]
pub async fn register(State(state): State<state::AppState>, Json(req): Json<UserReq>) -> Json<CreateUserRes> {
    let pool = state.db;

    let password_hash = hash(req.password.as_bytes(), DEFAULT_COST).unwrap();

    let user_id = sqlx::query_scalar!(
        r#"INSERT INTO users (full_name, email, password_hash) VALUES ($1, $2, $3) returning id"#,
        req.full_name,
        req.email,
        password_hash
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

    let token = generate_jwt_token(user_id).unwrap();
    let res = CreateUserRes {
        account_id: account_id,
        full_name: req.full_name,
        email: req.email,
        account_type: req.account_type,
        token: token
    };

    Json(res)
}

fn generate_jwt_token(user_id: Uuid) -> Result<String, StatusCode> {
    let expiration = time::OffsetDateTime::now_utc() + time::Duration::days(7);
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.unix_timestamp() as usize,
    };

    let secret_key = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY is not defined");

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_bytes())
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(token)
}


#[axum::debug_handler]
pub async fn login(
    State(state): State<state::AppState>,
    Json(req): Json<LoginReq>
) -> Result<Json<LoginRes>, StatusCode> {
    let pool = state.db;

    // Get user from database
    let user = sqlx::query!(
        r#"SELECT id, full_name, email, created_at, password_hash FROM users WHERE email = $1"#,
        req.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify password
    if !verify(&req.password, &user.password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Create JWT token
    let token = generate_jwt_token(user.id)?;

    let user = User {
        id: user.id,
        full_name: user.full_name,
        email: user.email,
        created_at: user.created_at,
    };

    let account_id = sqlx::query_scalar!(
        "SELECT id FROM accounts WHERE user_id = $1",
        user.id
    ).fetch_one(&pool).await.unwrap();

    Ok(Json(LoginRes { token, account_id: account_id.to_string(), full_name: user.full_name }))
}

pub async fn update_profile(
    State(state): State<state::AppState>,
    Json(req): Json<UpdateProfileReq>
) -> Result<Json<CreateUserRes>, StatusCode> {
    let pool = state.db;

    // Get current user data
    let current_user = sqlx::query!(
        r#"SELECT id, full_name, email, password_hash FROM users WHERE email = $1"#,
        req.email
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update only provided fields
    let new_name = req.full_name.unwrap_or(current_user.full_name);
    let new_email = req.email.unwrap_or(current_user.email);
    let new_password_hash = if let Some(new_password) = req.password {
        hash(&new_password, DEFAULT_COST).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        current_user.password_hash
    };

    let updated_user = sqlx::query_as!(
        User,
        r#"
        UPDATE users 
        SET full_name = $1,
            email = $2,
            password_hash = $3
        WHERE id = $4
        RETURNING id, full_name, email, created_at
        "#,
        new_name,
        new_email,
        new_password_hash,
        current_user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = generate_jwt_token(current_user.id).unwrap();

    let res = CreateUserRes {
        account_id: current_user.id,
        full_name: updated_user.full_name,
        email: updated_user.email,
        account_type: Types::Savings,
        token: token
    };
    Ok(Json(res))
}