use axum::{Json, Router, extract::State, routing::get};

use super::query::Query;
use crate::{entity::logs, error::Result, extractors::AdminUser, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_logs))
}

async fn get_logs(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> Result<Json<Vec<logs::Model>>> {
    Ok(Json(Query::get_all_logs(&state).await?))
}
