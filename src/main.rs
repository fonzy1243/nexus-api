mod entity;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

use sea_orm::{Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use tokio::signal::ctrl_c;

// Application state
#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

async fn shutdown_signal() {
    ctrl_c().await.expect("Failed to listen for Ctrl-C");
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    if dotenvy::from_filename(".env.local").is_ok() {
        tracing::info!("Loaded .env.local");
    } else if dotenvy::dotenv().is_ok() {
        tracing::info!("Loaded .env");
    } else {
        tracing::info!("No .env file found, using environment variables");
    }

    let connection_str = dotenvy::var("DATABASE_URL")?;

    let db = Database::connect(connection_str).await?;
    db.get_schema_registry("nexus-api::entity::*")
        .sync(&db)
        .await?;

    let state = AppState { db };

    let app = Router::new().route("/", get(root)).with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Fatal error: {e}");
        std::process::exit(1);
    }
}

async fn root() -> &'static str {
    "Hello, world!"
}
