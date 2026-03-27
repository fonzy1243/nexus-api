use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use super::query::{CommentWithReplies, ListParams, PostSummary, Query as PostQuery};
use crate::{error::Result, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_posts))
        .route("/{id}/comments", get(get_post_comments))
}

async fn get_all_posts(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<PostSummary>>> {
    Ok(Json(PostQuery::get_all_posts(&state, params).await?))
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
