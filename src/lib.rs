use axum::{
    routing::{get, post},
    Router,
};

pub mod api;
pub mod middleware;
pub mod state;

pub fn app(state: state::AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/v1/user/register", post(api::user::register))
        .route("/api/v1/user/updateProfile", get(api::user::update_profile))
        .route("/api/v1/user/login", post(api::user::login))
        .route("/api/v1/transaction/create", post(api::transaction::create))
        .route("/api/v1/transaction/all", get(api::transaction::get_all))
        .route("/api/v1/transaction/query", get(api::transaction::query))
        .route("/api/v1/account/checkBalance", get(api::account::check_balance))
        .route("/api/v1/account/updateBalance", post(api::account::update_balance))
        .with_state(state)
        .layer(axum::middleware::from_fn(
            middleware::auth::auth,
        ))
}