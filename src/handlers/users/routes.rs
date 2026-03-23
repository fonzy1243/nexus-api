use axum::{
    Json, Router,
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, get_service, patch, post},
};
use serde::{Deserialize, Serialize};

use super::mutation::{
    AuthResponse, ChangePasswordInput, ChangeUsernameInput, LoginInput, Mutation, RefreshInput,
    RegisterInput,
};
use crate::{error::Result, extractors::AuthUser, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/me/username", patch(change_username))
        .route("/me/password", patch(change_password))
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

async fn refresh(
    State(state): State<AppState>,
    Json(input): Json<RefreshInput>,
) -> Result<Json<AuthResponse>> {
    let res = Mutation::refresh(&state, input).await?;
    Ok(Json(res))
}

async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<RefreshInput>,
) -> Result<Json<String>> {
    Mutation::logout(&state, auth.id, input).await?;
    Ok(Json("Logged out".into()))
}

async fn change_username(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<ChangeUsernameInput>,
) -> Result<Json<String>> {
    Mutation::change_username(&state, auth.id, input).await?;
    Ok(Json("Username updated".into()))
}

async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<ChangePasswordInput>,
) -> Result<Json<String>> {
    Mutation::change_password(&state, auth.id, input).await?;
    Ok(Json("Password updated".into()))
}
