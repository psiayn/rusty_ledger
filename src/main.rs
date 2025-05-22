use dotenv::dotenv;
use rusty_ledger::{app, state};
use sqlx::postgres::PgPoolOptions;


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not defined");

    let db = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let state = state::AppState{db};

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app(state)).await.unwrap();
    Ok(())
}