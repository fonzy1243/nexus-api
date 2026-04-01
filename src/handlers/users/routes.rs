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
use uuid::Uuid;

use super::mutation::{
    AuthResponse, ChangePasswordInput, ChangeUsernameInput, LoginInput, Mutation, RefreshInput,
    RegisterInput, ResetPasswordInput, SetSecurityQuestionInput,
};
use super::query::{Params, PostSummary, Query as UserQuery, UserSummary};
use crate::{
    entity::comments,
    error::{AppError, Result},
    extractors::AuthUser,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // public
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/reset-password", post(reset_password))
        .route("/auth/security-question", post(get_security_question))
        .route("/{id}", get(get_user_by_id))
        .route("/{id}/posts", get(get_user_posts))
        .route("/{id}/comments", get(get_user_comments))
        // protected - uses AuthUser
        .route("/auth/logout", post(logout))
        .route("/me/username", patch(change_username))
        .route("/me/password", patch(change_password))
        .route("/me/security-question", post(set_security_question))
}

async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(input): Json<RegisterInput>,
) -> Result<impl IntoResponse> {
    let res = Mutation::register(&state, input).await?;

    let cookie = Cookie::build(("refresh_token", res.refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .path("/users/auth")
        .max_age(time::Duration::days(30))
        .build();

    Ok((
        jar.add(cookie),
        Json(json!({
            "access_token": res.access_token,
            "user_id": res.user_id,
            "username": res.username,
            "role": res.role,
        })),
    ))
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
        .path("/users/auth")
        .max_age(time::Duration::days(30))
        .build();

    Ok((
        jar.add(cookie),
        Json(json!({
            "access_token": res.access_token,
            "user_id": res.user_id,
            "username": res.username,
            "role": res.role,
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
        .path("/users/auth")
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
        .path("/users/auth")
        .max_age(time::Duration::seconds(0))
        .build();

    Ok((jar.add(cookie), Json(json!({ "message": "Logged out" }))))
}

async fn change_username(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<ChangeUsernameInput>,
) -> Result<Json<String>> {
    Mutation::change_username(&state, &auth, input).await?;
    Ok(Json("Username updated".into()))
}

async fn reset_password(
    State(state): State<AppState>,
    Json(input): Json<ResetPasswordInput>,
) -> Result<Json<String>> {
    Mutation::reset_password(&state, input).await?;
    Ok(Json("Password reset".into()))
}

async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<ChangePasswordInput>,
) -> Result<Json<String>> {
    Mutation::change_password(&state, auth.id, input).await?;
    Ok(Json("Password updated".into()))
}

async fn get_security_question(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<String>> {
    let email = body["email"]
        .as_str()
        .ok_or(AppError::BadRequest("Email required".into()))?
        .to_string();

    let question = Mutation::get_security_question(&state, email).await?;
    Ok(Json(question))
}

async fn set_security_question(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<SetSecurityQuestionInput>,
) -> Result<Json<String>> {
    Mutation::set_security_question(&state, auth.id, input).await?;
    Ok(Json("Security question set".into()))
}

async fn get_user_by_id(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserSummary>> {
    let user = UserQuery::find_user_by_id(&state, user_id).await?;
    Ok(Json(user))
}

async fn get_user_posts(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<Params>,
) -> Result<Json<Vec<PostSummary>>> {
    let posts = UserQuery::get_user_posts(&state, id, params).await?;
    Ok(Json(posts))
}

async fn get_user_comments(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<Params>,
) -> Result<Json<Vec<comments::Model>>> {
    let comments = UserQuery::get_user_comments(&state, id, params).await?;
    Ok(Json(comments))
}
