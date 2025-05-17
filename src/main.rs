use axum::{
    routing::get,
    Router,
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;


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


    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
