use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
};
use uuid::Uuid;

use super::{
    mutation::{CreatePostInput, Mutation as PostMutation, UpdatePostInput},
    query::{CommentWithReplies, ListParams, PostSummary, Query as PostQuery},
};
use crate::{
    entity::comments,
    entity::users::UserRole,
    error::Result,
    extractors::AuthUser,
    handlers::comments::mutation::{
        CreateCommentInput, Mutation as CommentMutation, UpdateCommentInput,
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_posts).post(create_post))
        .route("/{id}", patch(update_post).delete(delete_post))
        .route(
            "/{id}/comments",
            get(get_post_comments).post(create_comment),
        )
        .route(
            "/{id}/comments/{comment_id}",
            patch(update_comment).delete(delete_comment),
        )
}

async fn get_all_posts(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<PostSummary>>> {
    Ok(Json(PostQuery::get_all_posts(&state, params).await?))
}

async fn create_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CreatePostInput>,
) -> Result<(StatusCode, Json<PostSummary>)> {
    let post = PostMutation::create_post(&state, auth.id, input).await?;
    Ok((
        StatusCode::CREATED,
        Json(PostQuery::get_post_by_id(&state, post.id).await?),
    ))
}

async fn update_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(post_id): Path<Uuid>,
    Json(input): Json<UpdatePostInput>,
) -> Result<Json<PostSummary>> {
    let is_admin = auth.role == UserRole::Admin;
    PostMutation::update_post(&state, auth.id, post_id, input, is_admin).await?;
    Ok(Json(PostQuery::get_post_by_id(&state, post_id).await?))
}

async fn delete_post(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(post_id): Path<Uuid>,
) -> Result<StatusCode> {
    let is_admin = auth.role == UserRole::Admin;
    PostMutation::delete_post(&state, auth.id, post_id, is_admin).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_post_comments(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<CommentWithReplies>>> {
    Ok(Json(
        PostQuery::get_post_comments(&state, post_id, params).await?,
    ))
}

async fn create_comment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(post_id): Path<Uuid>,
    Json(input): Json<CreateCommentInput>,
) -> Result<(StatusCode, Json<comments::Model>)> {
    let comment = CommentMutation::create_comment(&state, auth.id, post_id, input).await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

async fn update_comment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((_, comment_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateCommentInput>,
) -> Result<Json<comments::Model>> {
    let is_admin = auth.role == UserRole::Admin;
    let comment =
        CommentMutation::update_comment(&state, auth.id, comment_id, input, is_admin).await?;
    Ok(Json(comment))
}

async fn delete_comment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((_, comment_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    let is_admin = auth.role == UserRole::Admin;
    CommentMutation::delete_comment(&state, auth.id, comment_id, is_admin).await?;
    Ok(StatusCode::NO_CONTENT)
}
