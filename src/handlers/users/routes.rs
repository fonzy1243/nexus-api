use axum::{
    Json, Router,
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service, patch, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use super::mutation::{
    AuthResponse, ChangePasswordInput, ChangeUsernameInput, LoginInput, Mutation, RefreshInput,
    RegisterInput, ResetPasswordInput, SetSecurityQuestionInput, UpdateRoleInput,
};
use super::query::{Params, PostSummary, Query as UserQuery, UserSummary};
use crate::{
    entity::comments,
    error::{AppError, Result},
    extractors::{AdminUser, AuthUser},
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
        // .route("/{id}", get(get_user_by_id))
        .route("/{username}", get(get_user_by_username))
        .route("/{id}/posts", get(get_user_posts))
        .route("/{id}/comments", get(get_user_comments))
        // protected - uses AuthUser or AdminUser
        .route("/auth/logout", post(logout))
        .route("/me/username", patch(change_username))
        .route("/me/password", patch(change_password))
        .route("/me/security-question", post(set_security_question))
        .route("/admin/role", post(make_admin).delete(remove_admin))
}

async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(input): Json<RegisterInput>,
) -> Result<impl IntoResponse> {
    let (res, raw_refresh) = Mutation::register(&state, input).await?;

    let is_prod = !cfg!(debug_assertions);
    let cookie = Cookie::build(("refresh_token", raw_refresh))
        .http_only(true)
        .secure(is_prod)
        .same_site(if is_prod {
            SameSite::None
        } else {
            SameSite::Lax
        })
        .path("/")
        .max_age(time::Duration::days(30))
        .build();

    Ok((
        jar.add(cookie),
        Json(json!({
            "access_token": res.access_token,
            "refresh_token_id": res.refresh_token_id,
            "user_id": res.user_id,
            "username": res.username,
            "role": res.role
        })),
    ))
}

async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(input): Json<LoginInput>,
) -> Result<impl IntoResponse> {
    let (res, raw_refresh) = Mutation::login(&state, input).await?;

    let is_prod = !cfg!(debug_assertions);

    let cookie = Cookie::build(("refresh_token", raw_refresh))
        .http_only(true)
        .secure(is_prod)
        .same_site(if is_prod {
            SameSite::None
        } else {
            SameSite::Lax
        })
        .path("/")
        .max_age(time::Duration::days(30))
        .build();

    Ok((
        jar.add(cookie),
        Json(json!({
            "access_token": res.access_token,
            "refresh_token_id": res.refresh_token_id,
            "user_id": res.user_id,
            "username": res.username,
            "role": res.role
        })),
    ))
}

async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse> {
    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let refresh_token_id = body["refresh_token_id"]
        .as_str()
        .ok_or(AppError::BadRequest("refresh_token_id required".into()))?
        .to_string();

    let (res, raw_refresh) = Mutation::refresh(
        &state,
        RefreshInput {
            refresh_token,
            refresh_token_id,
        },
    )
    .await?;

    let is_prod = !cfg!(debug_assertions);
    let cookie = Cookie::build(("refresh_token", raw_refresh))
        .http_only(true)
        .secure(is_prod)
        .same_site(if is_prod {
            SameSite::None
        } else {
            SameSite::Lax
        })
        .path("/")
        .max_age(time::Duration::days(30))
        .build();

    Ok((
        jar.add(cookie),
        Json(json!({
            "access_token": res.access_token,
            "refresh_token_id": res.refresh_token_id,
            "user_id": res.user_id,
            "username": res.username,
            "role": res.role
        })),
    ))
}

async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
    jar: CookieJar,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse> {
    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let refresh_token_id = body["refresh_token_id"]
        .as_str()
        .ok_or(AppError::BadRequest("refresh_token_id required".into()))?
        .to_string();

    Mutation::logout(
        &state,
        auth.id,
        RefreshInput {
            refresh_token,
            refresh_token_id,
        },
    )
    .await?;

    let is_prod = !cfg!(debug_assertions);
    let cookie = Cookie::build(("refresh_token", "\0"))
        .path("/")
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

async fn get_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<UserSummary>> {
    let user = UserQuery::find_user_by_username(&state, username).await?;
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

async fn make_admin(
    State(state): State<AppState>,
    admin: AdminUser,
    Json(input): Json<UpdateRoleInput>,
) -> Result<StatusCode> {
    Mutation::make_admin(&state, &admin, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_admin(
    State(state): State<AppState>,
    admin: AdminUser,
    Json(input): Json<UpdateRoleInput>,
) -> Result<StatusCode> {
    Mutation::remove_admin(&state, &admin, input).await?;
    Ok(StatusCode::NO_CONTENT)
}
