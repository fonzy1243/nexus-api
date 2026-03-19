use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

use sea_orm::{Database, DatabaseConnection};
use serde::{Deserialize, Serialize};

// Application state
#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let connection_str = dotenvy::var("CONNECTION_STRING")?;

    let db = Database::connect(connection_str).await?;

    let state = AppState { db };

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

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

async fn create_user(
    // parse request body as JSON into 'CreateUser' type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: 67,
        username: payload.username,
    };

    (StatusCode::CREATED, Json(user))
}

// input to 'create_user' handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// output to 'create_user' handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
