use axum::{Json, Router, extract::{Query, State}, routing::get};

use super::query::{Query as SearchQuery, SearchParams, SearchResults};
use crate::{error::Result, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(search))
}

async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResults>> {
    Ok(Json(SearchQuery::search(&state, params).await?))
}
