// use crate deps
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entity::{
        communities, communities::Entity as Communities, posts, posts::Entity as Posts, users,
        users::Entity as Users,
    },
    error::Result,
    state::AppState,
};

use super::super::posts::query::{ListParams, PostSummary, Query as PostQuery, SortBy, SortOrder};

#[derive(Deserialize)]
pub struct PagaParams {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Serialize)]
pub struct CommunitySummary {
    pub id: Uuid,
    pub name: String,
    pub logo: String,
    pub created_at: String,
}

pub struct Query;

impl Query {
    pub async fn get_all_communities(
        state: &AppState,
        params: PagaParams,
    ) -> Result<Vec<CommunitySummary>> {
        let page = params.page.unwrap_or(1).saturating_sub(1);
        let limit = Ord::min(params.limit.unwrap_or(20), 100);

        let rows = Communities::find()
            .order_by_asc(communities::Column::Name)
            .offset(page * limit)
            .limit(limit)
            .all(&state.db)
            .await?;

        Ok(rows
            .into_iter()
            .map(|c| CommunitySummary {
                id: c.id,
                name: c.name,
                logo: c.logo,
                created_at: c.created_at.to_string(),
            })
            .collect())
    }

    pub async fn get_community_posts(
        state: &AppState,
        community_id: Uuid,
        params: ListParams,
    ) -> Result<Vec<PostSummary>> {
        PostQuery::get_community_posts(state, community_id, params).await
    }
}
