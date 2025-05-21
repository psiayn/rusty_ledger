use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
mod api;
mod middleware;
pub mod state;


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not defined");

    let db = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let state = state::AppState{db};


    // build our application with a single route
    let app = Router::new()
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
        ));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
