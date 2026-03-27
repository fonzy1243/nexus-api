mod auth;
mod entity;
mod error;
mod extractors;
mod handlers;
mod logger;
mod state;

use state::AppState;

use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

use sea_orm::Database;
use tokio::signal::ctrl_c;

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
    let jwt_secret = dotenvy::var("JWT_SECRET")?;

    let db = Database::connect(connection_str).await?;
    db.get_schema_registry("nexus-api::entity::*")
        .sync(&db)
        .await?;

    let state = AppState { db, jwt_secret };

    let app = Router::new()
        .route("/", get(root))
        .nest("/users", handlers::users::routes::router())
        .nest("/posts", handlers::posts::routes::router())
        .nest("/communities", handlers::communities::routes::router())
        .nest("/logs", handlers::logs::routes::router())
        .with_state(state);

    let app = app.fallback(handler_404);

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

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Nothing to see here.")
}
