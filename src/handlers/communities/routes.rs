use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, patch, post},
};
use http::StatusCode;
use uuid::Uuid;

use super::query::{CommunitySummary, PagaParams, Query as CommunityQuery};
use crate::{
    error::Result,
    extractors::AuthUser,
    handlers::{
        communities::mutation::{
            CreateCommunityInput, Mutation as CommunityMutation, UpdateCommunityInput,
            UpdateModeratorInput,
        },
        posts::query::{ListParams, PostSummary},
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_communities).post(create_community))
        .route("/{id}", patch(update_community).delete(delete_community))
        .route("/{id}/posts", get(get_community_posts))
        .route("/{id}/mod-status", get(get_moderator_status))
        .route(
            "/{id}/moderators",
            post(make_moderator).delete(remove_moderator),
        )
        .route("/{id}/join", post(join_community).delete(leave_community))
}

async fn get_all_communities(
    State(state): State<AppState>,
    Query(params): Query<PagaParams>,
) -> Result<Json<Vec<CommunitySummary>>> {
    Ok(Json(
        CommunityQuery::get_all_communities(&state, params).await?,
    ))
}

async fn create_community(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CreateCommunityInput>,
) -> Result<(StatusCode, Json<CommunitySummary>)> {
    let community = CommunityMutation::create_community(&state, auth.id, input).await?;
    Ok((
        StatusCode::CREATED,
        Json(CommunitySummary {
            id: community.id,
            name: community.name,
            logo: community.logo,
            created_at: community.created_at.to_string(),
        }),
    ))
}

async fn update_community(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(community_id): Path<Uuid>,
    Json(input): Json<UpdateCommunityInput>,
) -> Result<Json<CommunitySummary>> {
    let community = CommunityMutation::update_community(&state, &auth, community_id, input).await?;
    Ok(Json(CommunitySummary {
        id: community.id,
        name: community.name,
        logo: community.logo,
        created_at: community.created_at.to_string(),
    }))
}

async fn delete_community(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(community_id): Path<Uuid>,
) -> Result<StatusCode> {
    CommunityMutation::delete_community(&state, &auth, community_id).await?;
    Ok(StatusCode::NO_CONTENT)
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

async fn get_moderator_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(community_id): Path<Uuid>,
) -> Result<Json<bool>> {
    Ok(Json(
        CommunityQuery::is_moderator(&state, auth.id, community_id).await?,
    ))
}

async fn make_moderator(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(community_id): Path<Uuid>,
    Json(input): Json<UpdateModeratorInput>,
) -> Result<StatusCode> {
    CommunityMutation::make_moderator(&state, &auth, community_id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_moderator(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(community_id): Path<Uuid>,
    Json(input): Json<UpdateModeratorInput>,
) -> Result<StatusCode> {
    CommunityMutation::remove_moderator(&state, &auth, community_id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn join_community(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(community_id): Path<Uuid>,
) -> Result<StatusCode> {
    CommunityMutation::join_community(&state, auth.id, community_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn leave_community(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(community_id): Path<Uuid>,
) -> Result<StatusCode> {
    CommunityMutation::leave_community(&state, auth.id, community_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
