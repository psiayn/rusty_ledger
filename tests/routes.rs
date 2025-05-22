use rusty_ledger::{app as create_app, state};
use axum::{
    body::Body,
    http::{self, Request, StatusCode, header},
};
use http_body_util::BodyExt; // for `collect`
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt; // for `oneshot`
use uuid::Uuid;
use bigdecimal::BigDecimal;
use std::str::FromStr;

// Helper function to create a test user and return their credentials
async fn create_test_user(pool: &PgPool, email: &str) -> (Uuid, Uuid, String) {
    // Clone pool to avoid ownership issues
    let pool_clone = pool.clone();
    let app = create_app(state::AppState { db: pool_clone });
    
    // Register a test user
    let register_response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/user/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "full_name": "Test User",
                        "email": email,
                        "password": "password123",
                        "account_type": "Savings"
                    })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(register_response.status(), StatusCode::OK);
    
    let register_body = register_response.into_body().collect().await.unwrap().to_bytes();
    let register_json: Value = serde_json::from_slice(&register_body).unwrap();
    let account_id = Uuid::parse_str(register_json["account_id"].as_str().unwrap()).unwrap();
    let token = register_json["token"].as_str().unwrap().to_string();
    
    // Get user_id from database directly
    let user_id = sqlx::query_scalar!(
        "SELECT user_id FROM accounts WHERE id = $1",
        account_id
    )
    .fetch_one(pool)
    .await
    .unwrap();
    
    (user_id, account_id, token)
}

// Helper function to seed initial balance
async fn seed_initial_balance(pool: &PgPool, account_id: Uuid, amount: &str) {
    sqlx::query!(
        "UPDATE account_balances SET balance = $1 WHERE account_id = $2",
        BigDecimal::from_str(amount).unwrap(),
        account_id
    )
    .execute(pool)
    .await
    .unwrap();
}

// Test for the root endpoint
#[sqlx::test]
async fn test_hello_world(pool: PgPool) {
    let app = create_app(state::AppState { db: pool });

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"Hello, World!");
}

// Test user registration
#[sqlx::test]
async fn test_user_register(pool: PgPool) {
    let app = create_app(state::AppState { db: pool });

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/user/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "full_name": "John Doe",
                        "email": "john@example.com",
                        "password": "password123",
                        "account_type": "Savings"
                    })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json.get("account_id").is_some());
    assert_eq!(json["full_name"], "John Doe");
    assert_eq!(json["email"], "john@example.com");
    assert_eq!(json["account_type"], "Savings");
}

// Test user login
#[sqlx::test]
async fn test_user_login(pool: PgPool) {
    // Create a test user first
    let app = create_app(state::AppState { db: pool.clone() });
    
    // Register a user
    app.oneshot(
        Request::builder()
            .method(http::Method::POST)
            .uri("/api/v1/user/register")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "full_name": "Jane Doe",
                    "email": "jane@example.com",
                    "password": "password123",
                    "account_type": "Current"
                })).unwrap(),
            ))
            .unwrap(),
    )
    .await
    .unwrap();
    
    // Create a new app instance to avoid ownership issues
    let app = create_app(state::AppState { db: pool });
    
    // Try to login
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/user/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "email": "jane@example.com",
                        "password": "password123"
                    })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json.get("token").is_some());
    assert!(json.get("account_id").is_some());
    assert_eq!(json["full_name"], "Jane Doe");
}

// Test update profile with authentication
#[sqlx::test]
async fn test_update_profile(pool: PgPool) {
    // Create test user and get token
    let (_, _, token) = create_test_user(&pool, "update@example.com").await;
    
    let app = create_app(state::AppState { db: pool });
    
    // Update profile
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/v1/user/updateProfile")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "full_name": "Updated Name",
                        "email": "update@example.com"
                    })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["full_name"], "Updated Name");
    assert_eq!(json["email"], "update@example.com");
}

// Test check balance with authentication
#[sqlx::test]
async fn test_check_balance(pool: PgPool) {
    // Create test user and get token
    let (_, account_id, token) = create_test_user(&pool, "balance@example.com").await;
    
    // Set initial balance
    seed_initial_balance(&pool, account_id, "100.00").await;
    
    let app = create_app(state::AppState { db: pool });
    
    // Check balance
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/api/v1/account/checkBalance?account_id={}", account_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["account_id"], account_id.to_string());
    // Check that the balance starts with the expected value, ignoring exact decimal formatting
    assert!(json["balance"].as_str().unwrap().starts_with("100"));
}

// Test update balance with authentication
#[sqlx::test]
async fn test_update_balance(pool: PgPool) {
    // Create test user and get token
    let (_, account_id, token) = create_test_user(&pool, "update_balance@example.com").await;
    
    let app = create_app(state::AppState { db: pool });
    
    // Update balance
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/account/updateBalance")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "account_id": account_id.to_string(),
                        "balance": "500.00"
                    })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["account_id"], account_id.to_string());
    // Check that the balance starts with the expected value, ignoring exact decimal formatting
    assert!(json["balance"].as_str().unwrap().starts_with("500"));
}

// Test create transaction with authentication
#[sqlx::test]
async fn test_create_transaction(pool: PgPool) {
    // Create two users
    let (_, from_account_id, token) = create_test_user(&pool, "from@example.com").await;
    let (_, to_account_id, _) = create_test_user(&pool, "to@example.com").await;
    
    // Set initial balances
    seed_initial_balance(&pool, from_account_id, "1000.00").await;
    seed_initial_balance(&pool, to_account_id, "500.00").await;
    
    let app = create_app(state::AppState { db: pool });
    
    // Create transaction
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/v1/transaction/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "from_account_id": from_account_id.to_string(),
                        "to_account_id": to_account_id.to_string(),
                        "amount": "200.00"
                    })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// Test get all transactions with authentication
#[sqlx::test]
async fn test_get_all_transactions(pool: PgPool) {
    // Create test user and transaction
    let (_, from_account_id, token) = create_test_user(&pool, "trans_all@example.com").await;
    let (_, to_account_id, _) = create_test_user(&pool, "trans_all_to@example.com").await;
    
    // Set initial balances and create a transaction
    seed_initial_balance(&pool, from_account_id, "1000.00").await;
    seed_initial_balance(&pool, to_account_id, "500.00").await;
    
    // Create a transaction
    let transaction_app = create_app(state::AppState { db: pool.clone() });
    
    transaction_app.oneshot(
        Request::builder()
            .method(http::Method::POST)
            .uri("/api/v1/transaction/create")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "from_account_id": from_account_id.to_string(),
                    "to_account_id": to_account_id.to_string(),
                    "amount": "100.00"
                })).unwrap(),
            ))
            .unwrap(),
    )
    .await
    .unwrap();
    
    // Create new app for the get request
    let app = create_app(state::AppState { db: pool });
    
    // Get all transactions
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/v1/transaction/all")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json.as_array().unwrap().len() > 0);
}

// Test query transactions with authentication
#[sqlx::test]
async fn test_query_transactions(pool: PgPool) {
    // Create test user and transaction
    let (_, from_account_id, token) = create_test_user(&pool, "trans_query@example.com").await;
    let (_, to_account_id, _) = create_test_user(&pool, "trans_query_to@example.com").await;
    
    // Set initial balances and create a transaction
    seed_initial_balance(&pool, from_account_id, "1000.00").await;
    seed_initial_balance(&pool, to_account_id, "500.00").await;
    
    // Create a transaction
    let transaction_app = create_app(state::AppState { db: pool.clone() });
    
    transaction_app.oneshot(
        Request::builder()
            .method(http::Method::POST)
            .uri("/api/v1/transaction/create")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "from_account_id": from_account_id.to_string(),
                    "to_account_id": to_account_id.to_string(),
                    "amount": "150.00"
                })).unwrap(),
            ))
            .unwrap(),
    )
    .await
    .unwrap();
    
    // Create new app for the query request
    let app = create_app(state::AppState { db: pool });
    
    // Query transactions
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/api/v1/transaction/query?account_id={}", from_account_id))
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["from_account_id"], from_account_id.to_string());
    assert_eq!(json["to_account_id"], to_account_id.to_string());
    // Check that the amount starts with the expected value, ignoring exact decimal formatting
    assert!(json["amount"].as_str().unwrap().starts_with("150"));
}

// Test unauthorized access
#[sqlx::test]
async fn test_unauthorized_access(pool: PgPool) {
    let app = create_app(state::AppState { db: pool });
    
    // Try to access a protected endpoint without token
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/v1/transaction/all")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// Test with invalid token
#[sqlx::test]
async fn test_invalid_token(pool: PgPool) {
    let app = create_app(state::AppState { db: pool });
    
    // Try to access a protected endpoint with invalid token
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/v1/transaction/all")
                .header(header::AUTHORIZATION, "Bearer invalid.token.here")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}