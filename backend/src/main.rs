use sqlx::postgres::PgPoolOptions;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let url = std::env::var("CROWD_DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let addr = std::env::var("CROWD_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(address = %addr, "API listening");
    axum::serve(listener, crowd_api::router(pool.clone()))
        .with_graceful_shutdown(shutdown())
        .await?;
    pool.close().await;
    Ok(())
}
async fn shutdown() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("install SIGTERM handler");
        tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
    }
    #[cfg(not(unix))]
    tokio::signal::ctrl_c()
        .await
        .expect("install Ctrl+C handler");
}
