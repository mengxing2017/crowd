//! Shared migration, worker and HTTP lifecycle for external and embedded databases.
use std::net::SocketAddr;

use sqlx::PgPool;

pub fn bind_address() -> Result<SocketAddr, std::net::AddrParseError> {
    std::env::var("CROWD_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()
}

/// The caller owns the pool and must close it on both success and startup failure.
pub async fn serve(pool: PgPool, address: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    serve_inner(pool, address, true).await
}

/// Local runner has its own temporary credentials and ignores production bootstrap secrets.
pub async fn serve_embedded(
    pool: PgPool,
    address: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    if !address.ip().is_loopback() {
        return Err("embedded API must bind to loopback".into());
    }
    serve_inner(pool, address, false).await
}

async fn serve_inner(
    pool: PgPool,
    address: SocketAddr,
    configured_bootstrap: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::migrate!("./migrations").run(&pool).await?;
    if configured_bootstrap {
        let username = std::env::var("CROWD_BOOTSTRAP_ADMIN_USERNAME")
            .ok()
            .filter(|v| !v.is_empty());
        let password = std::env::var("CROWD_BOOTSTRAP_ADMIN_PASSWORD")
            .ok()
            .filter(|v| !v.is_empty());
        match (username, password) {
            (Some(username), Some(password)) => {
                crate::community::south::adapter::postgres::Repository { pool: pool.clone() }
                    .bootstrap_staff(&username, &password)
                    .await
                    .map_err(|_| "administrator bootstrap failed; check configuration")?;
            }
            (None, None) => {}
            _ => return Err("administrator bootstrap requires both username and password".into()),
        }
    }
    let listener = tokio::net::TcpListener::bind(address).await?;
    let code_jobs = crate::community::south::adapter::postgres::Repository { pool: pool.clone() };
    let worker = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if code_jobs.retry_code_jobs().await.is_err() {
                tracing::warn!("promotion code retry queue temporarily unavailable");
            }
        }
    });
    tracing::info!(address = %listener.local_addr()?, "API listening");
    let result = axum::serve(listener, crate::router(pool))
        .with_graceful_shutdown(shutdown())
        .await;
    worker.abort();
    let _ = worker.await;
    result?;
    Ok(())
}

pub async fn shutdown() {
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
