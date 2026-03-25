use axum::{
    Json, Router,
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service, patch, post},
};
use axum_extra::extract::cookie::{self, Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::mutation::{
    AuthResponse, ChangePasswordInput, ChangeUsernameInput, LoginInput, Mutation, RefreshInput,
    RegisterInput,
};
use crate::{
    error::{AppError, Result},
    extractors::AuthUser,
    state::AppState,
};

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
    jar: CookieJar,
    Json(input): Json<LoginInput>,
) -> Result<impl IntoResponse> {
    let res = Mutation::login(&state, input).await?;

    let cookie = Cookie::build(("refresh_token", res.refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .path("/api/users/auth")
        .max_age(time::Duration::days(30))
        .build();

    Ok((
        jar.add(cookie),
        Json(json!({
            "access_token": res.access_token,
            "user_id": res.user_id,
            "username": res.username,
        })),
    ))
}

async fn refresh(State(state): State<AppState>, jar: CookieJar) -> Result<impl IntoResponse> {
    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;
    let res = Mutation::refresh(&state, RefreshInput { refresh_token }).await?;

    let cookie = Cookie::build(("refresh_token", res.refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .path("/api/users/auth")
        .max_age(time::Duration::days(30))
        .build();

    Ok((
        jar.add(cookie),
        Json(json!({
            "access_token": res.access_token,
        })),
    ))
}

async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
    jar: CookieJar,
) -> Result<impl IntoResponse> {
    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;
    Mutation::logout(&state, auth.id, RefreshInput { refresh_token }).await?;

    let cookie = Cookie::build(("refresh_token", ""))
        .path("/api/users/auth")
        .max_age(time::Duration::seconds(0))
        .build();

    Ok((jar.add(cookie), Json(json!({ "message": "Logged out" }))))
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
