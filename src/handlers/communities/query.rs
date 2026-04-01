// use crate deps
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entity::{
        communities::{self, Entity as Communities},
        posts::{self, Entity as Posts},
        subscriptions::{self, Entity as Subscriptions, SubRole},
        users::{self, Entity as Users},
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
    pub member_count: u64,
    pub post_count: u64,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct CommunityMember {
    pub id: Uuid,
    pub username: String,
    pub role: SubRole,
    pub joined_at: String,
}

pub struct Query;

impl Query {
    pub async fn is_moderator(state: &AppState, user_id: Uuid, community_id: Uuid) -> Result<bool> {
        let sub = Subscriptions::find()
            .filter(subscriptions::Column::CommunityId.eq(community_id))
            .filter(subscriptions::Column::SubscriberId.eq(user_id))
            .filter(subscriptions::Column::Role.eq(SubRole::Moderator))
            .one(&state.db)
            .await?;

        Ok(sub.is_some())
    }

    pub async fn is_member(state: &AppState, user_id: Uuid, community_id: Uuid) -> Result<bool> {
        let sub = Subscriptions::find()
            .filter(subscriptions::Column::CommunityId.eq(community_id))
            .filter(subscriptions::Column::SubscriberId.eq(user_id))
            .one(&state.db)
            .await?;

        Ok(sub.is_some())
    }

    pub async fn get_community_summary(
        state: &AppState,
        c: communities::Model,
    ) -> Result<CommunitySummary> {
        let member_count = Subscriptions::find()
            .filter(subscriptions::Column::CommunityId.eq(c.id))
            .count(&state.db)
            .await?;

        let post_count = Posts::find()
            .filter(posts::Column::CommunityId.eq(c.id))
            .count(&state.db)
            .await?;

        Ok(CommunitySummary {
            id: c.id,
            name: c.name,
            logo: c.logo,
            member_count,
            post_count,
            created_at: c.created_at.to_string(),
        })
    }

    pub async fn get_all_communities(
        state: &AppState,
        params: PagaParams,
    ) -> Result<Vec<CommunitySummary>> {
        let page = params.page.unwrap_or(1).saturating_sub(1);
        let limit = Ord::min(params.limit.unwrap_or(20), 100);

        let communities = Communities::find()
            .order_by_asc(communities::Column::Name)
            .offset(page * limit)
            .limit(limit)
            .all(&state.db)
            .await?;

        let mut summaries = Vec::new();
        for c in communities {
            summaries.push(Self::get_community_summary(state, c).await?);
        }

        Ok(summaries)
    }

    pub async fn get_community_posts(
        state: &AppState,
        community_id: Uuid,
        params: ListParams,
    ) -> Result<Vec<PostSummary>> {
        PostQuery::get_community_posts(state, community_id, params).await
    }

    pub async fn get_community_members(
        state: &AppState,
        community_id: Uuid,
    ) -> Result<Vec<CommunityMember>> {
        let rows = Subscriptions::find()
            .filter(subscriptions::Column::CommunityId.eq(community_id))
            .find_also_related(Users)
            .all(&state.db)
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(sub, user)| {
                let user = user?;
                Some(CommunityMember {
                    id: user.id,
                    username: user.username,
                    role: sub.role,
                    joined_at: sub.created_at.to_string(),
                })
            })
            .collect())
    }
}
