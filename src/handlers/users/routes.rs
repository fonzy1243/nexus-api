use axum::{
    Json, Router,
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, get_service, post},
};
use serde::{Deserialize, Serialize};

use super::mutation::{AuthResponse, LoginInput, Mutation, RegisterInput};
use crate::{error::Result, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}

async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterInput>,
) -> Result<Json<AuthResponse>> {
    let res = Mutation::register(&state, input).await?;
    Ok(Json(res))
}

async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<Json<AuthResponse>> {
    let res = Mutation::login(&state, input).await?;
    Ok(Json(res))
}
