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
pub async fn register(
    State(state): State<state::AppState>, 
    Json(req): Json<UserReq>
) -> Result<Json<CreateUserRes>, (StatusCode, String)> {
    let pool = state.db;

    // Check if the email already exists
    let existing_user = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
        req.email
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    if existing_user.unwrap_or(false) {
        return Err((StatusCode::CONFLICT, format!("User with email {} already exists", req.email)));
    }

    let password_hash = hash(req.password.as_bytes(), DEFAULT_COST)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to hash password: {}", e)))?;

    let user_id = sqlx::query_scalar!(
        r#"INSERT INTO users (full_name, email, password_hash) VALUES ($1, $2, $3) returning id"#,
        req.full_name,
        req.email,
        password_hash
    ).fetch_one(&pool).await
     .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create user: {}", e)))?;
    
    println!("user_id = {user_id}");
    
    let account_id = sqlx::query_scalar!(
        "INSERT INTO accounts (user_id, account_type) VALUES ($1, $2) returning id",
        user_id,
        req.account_type.to_string()
    ).fetch_one(&pool).await
     .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create account: {}", e)))?;
    
    println!("account_id = {account_id}");
    
    let balance = sqlx::query_scalar!(
        "INSERT INTO account_balances (account_id) VALUES ($1) returning balance",
        account_id
    ).fetch_one(&pool).await
     .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create account balance: {}", e)))?;
    
    println!("balance = {balance}");

    let token = generate_jwt_token(user_id)
        .map_err(|e| (e, "Failed to generate authentication token".to_string()))?;
    
    let res = CreateUserRes {
        account_id,
        full_name: req.full_name,
        email: req.email,
        account_type: req.account_type,
        token
    };

    Ok(Json(res))
}

fn generate_jwt_token(user_id: Uuid) -> Result<String, StatusCode> {
    let expiration = time::OffsetDateTime::now_utc() + time::Duration::days(7);
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.unix_timestamp() as usize,
    };

    let secret_key = std::env::var("JWT_SECRET_KEY")
        .map_err(|_| {
            eprintln!("JWT_SECRET_KEY environment variable not set");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_bytes())
    ).map_err(|e| {
        eprintln!("Failed to encode JWT token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(token)
}


#[axum::debug_handler]
pub async fn login(
    State(state): State<state::AppState>,
    Json(req): Json<LoginReq>
) -> Result<Json<LoginRes>, (StatusCode, String)> {
    let pool = state.db;

    // Get user from database
    let user = sqlx::query!(
        r#"SELECT id, full_name, email, created_at, password_hash FROM users WHERE email = $1"#,
        req.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
    .ok_or((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()))?;

    // Verify password
    if !verify(&req.password, &user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to verify password: {}", e)))? {
        return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
    }

    // Create JWT token
    let token = generate_jwt_token(user.id)
        .map_err(|e| (e, "Failed to generate authentication token".to_string()))?;

    let user = User {
        id: user.id,
        full_name: user.full_name,
        email: user.email,
        created_at: user.created_at,
    };

    let account_id = sqlx::query_scalar!(
        "SELECT id FROM accounts WHERE user_id = $1",
        user.id
    ).fetch_one(&pool).await
     .map_err(|e| match e {
        sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, format!("No account found for user {}", user.id)),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch account: {}", e))
     })?;

    Ok(Json(LoginRes { 
        token, 
        account_id: account_id.to_string(), 
        full_name: user.full_name 
    }))
}

pub async fn update_profile(
    State(state): State<state::AppState>,
    Json(req): Json<UpdateProfileReq>
) -> Result<Json<CreateUserRes>, (StatusCode, String)> {
    let pool = state.db;

    // At least one field must be provided for update
    if req.full_name.is_none() && req.email.is_none() && req.password.is_none() && req.account_type.is_none() {
        return Err((StatusCode::BAD_REQUEST, "At least one field must be provided for update".to_string()));
    }

    // Check if email exists
    if req.email.is_none() {
        return Err((StatusCode::BAD_REQUEST, "Email is required to identify the user".to_string()));
    }

    // Get current user data
    let current_user = sqlx::query!(
        r#"SELECT id, full_name, email, password_hash FROM users WHERE email = $1"#,
        req.email
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, format!("User with email {} not found", req.email.as_ref().unwrap())),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    // Update only provided fields
    let new_name = req.full_name.unwrap_or(current_user.full_name);
    let new_email = req.email.unwrap_or(current_user.email);
    let new_password_hash = if let Some(new_password) = req.password {
        hash(&new_password, DEFAULT_COST)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to hash password: {}", e)))?
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
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update user: {}", e)))?;

    let token = generate_jwt_token(current_user.id)
        .map_err(|e| (e, "Failed to generate authentication token".to_string()))?;

    // Get account type
    let account_type = if let Some(new_type) = req.account_type {
        // Update account type if provided
        sqlx::query!(
            "UPDATE accounts SET account_type = $1 WHERE user_id = $2",
            new_type.to_string(),
            current_user.id
        )
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update account type: {}", e)))?;
        
        new_type
    } else {
        // Get current account type
        let account_type_str = sqlx::query_scalar!(
            "SELECT account_type FROM accounts WHERE user_id = $1",
            current_user.id
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch account type: {}", e)))?;
        
        match account_type_str.as_str() {
            "savings" => Types::Savings,
            "current" => Types::Current,
            "salary" => Types::Salary,
            "fd" => Types::FD,
            "rd" => Types::RD,
            _ => Types::Savings // Default to Savings if unknown
        }
    };

    let res = CreateUserRes {
        account_id: current_user.id,
        full_name: updated_user.full_name,
        email: updated_user.email,
        account_type,
        token
    };
    
    Ok(Json(res))
}