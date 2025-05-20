use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
mod api;
pub mod state;


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv().ok();

    let postgres_db = std::env::var("POSTGRES_DB").expect("POSTGRES_DB is not defined");
    let postgres_schema = std::env::var("POSTGRES_SCHEMA").expect("POSTGRES_SCHEMA is not defined");
    let postgres_username = std::env::var("POSTGRES_USER").expect("POSTGRES_USER is not defined");
    let postgres_password = std::env::var("POSTGRES_PASS").expect("POSTGRES_PASS is not defined");
    let postgres_hostname = std::env::var("POSTGRES_HOST").expect("POSTGRES_HOST is not defined");

    let connection_string = format!("postgres://{postgres_username}:{postgres_password}@{postgres_hostname}/{postgres_db}?currentSchema={postgres_schema}");

    let db = PgPoolOptions::new().max_connections(5).connect(&connection_string).await?;

    let state = state::AppState{db};


    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/v1/user/register", post(api::user::register))
        .route("/api/v1/user/updateProfile", get(api::user::update_profile))
        .route("/api/v1/user/delete", get(api::user::delete))
        .route("/api/v1/user/login", get(api::user::login))
        .route("/api/v1/transaction/create", post(api::transaction::create))
        .route("/api/v1/transaction/all", get(api::transaction::get_all))
        .route("/api/v1/transaction/query", get(api::transaction::query))
        .route("/api/v1/account/checkBalance", get(api::account::check_balance))
        .route("/api/v1/account/updateBalance", post(api::account::update_balance))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
