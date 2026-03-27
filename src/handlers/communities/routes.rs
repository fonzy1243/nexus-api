use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use uuid::Uuid;

use super::query::{CommunitySummary, PagaParams, Query as CommunityQuery};
use crate::{
    error::Result,
    handlers::posts::query::{ListParams, PostSummary},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_communities))
        .route("/:id/posts", get(get_community_posts))
}

async fn get_all_communities(
    State(state): State<AppState>,
    Query(params): Query<PagaParams>,
) -> Result<Json<Vec<CommunitySummary>>> {
    Ok(Json(
        CommunityQuery::get_all_communities(&state, params).await?,
    ))
}

async fn get_community_posts(
    State(state): State<AppState>,
    Path(community_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<PostSummary>>> {
    Ok(Json(
        CommunityQuery::get_community_posts(&state, community_id, params).await?,
    ))
}
