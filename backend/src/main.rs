mod database_config;

use sqlx::postgres::PgPoolOptions;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let address = crowd_api::runtime::bind_address()?;
    let options = database_config::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;
    let result = crowd_api::runtime::serve(pool.clone(), address).await;
    pool.close().await;
    result
}
