//! Development-only PostgreSQL lifecycle. Does not read external database settings.
use std::{net::SocketAddr, path::Path, time::Duration};

use postgresql_embedded::{BOOTSTRAP_SUPERUSER, PostgreSQL, SettingsBuilder, V17};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "crowd_api=info,embedded_postgres=info".into()),
        )
        .init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let test_business = match args.as_slice() {
        [] => false,
        [arg] if arg == "--test-business" => true,
        _ => return Err("usage: embedded_postgres [--test-business]".into()),
    };
    let address = crowd_api::runtime::bind_address()?;
    if !address.ip().is_loopback() {
        return Err("embedded development API must bind to a loopback address".into());
    }
    let settings = SettingsBuilder::new()
        .host("127.0.0.1")
        .port(0)
        .username(BOOTSTRAP_SUPERUSER)
        .temporary(true)
        .timeout(Some(Duration::from_secs(90)))
        .version(V17.clone())
        .build();
    let mut postgres = PostgreSQL::new(settings);
    tracing::info!("preparing temporary PostgreSQL 17; first run may download server binaries");
    postgres.setup().await?;
    // Stop on failed start too; the library owns temporary data/password cleanup on drop.
    let result = async {
        postgres.start().await?;
        tracing::info!(
            database_port = postgres.settings().port,
            data_directory = %postgres.settings().data_dir.display(),
            "embedded PostgreSQL started; data will be removed on exit"
        );
        if test_business {
            run_business_tests(&postgres).await
        } else {
            run_api(&postgres, address).await
        }
    }
    .await;
    let stop_result = postgres.stop().await;
    result?;
    stop_result?;
    Ok(())
}

async fn run_api(
    postgres: &PostgreSQL,
    address: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    const DATABASE: &str = "crowd_dev";
    postgres.create_database(DATABASE).await?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&postgres.settings().url(DATABASE))
        .await?;
    let result = crowd_api::runtime::serve(pool.clone(), address).await;
    pool.close().await;
    result
}

async fn run_business_tests(postgres: &PostgreSQL) -> Result<(), Box<dyn std::error::Error>> {
    const DATABASE: &str = "crowd_test_embedded";
    postgres.create_database(DATABASE).await?;
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let mut child = tokio::process::Command::new("cargo")
        .args(["test", "--locked", "--manifest-path"])
        .arg(manifest)
        .args([
            "--test",
            "community_postgres",
            "--",
            "--ignored",
            "--nocapture",
        ])
        // Always override inherited test URLs; never forward an external application URL.
        .env_remove("CROWD_DATABASE_URL")
        .env(
            "CROWD_TEST_DATABASE_URL",
            postgres
                .settings()
                .url(DATABASE)
                .replacen("postgresql://", "postgres://", 1),
        )
        .env("CROWD_PROMOTION_ORIGIN", "https://crowd.example.test")
        .kill_on_drop(true)
        .spawn()?;
    let status = tokio::select! {
        result = child.wait() => result?,
        () = crowd_api::runtime::shutdown() => {
            child.start_kill()?;
            let _ = child.wait().await;
            return Err("embedded business tests interrupted".into());
        }
    };
    if !status.success() {
        return Err("embedded business integration tests failed".into());
    }
    Ok(())
}
