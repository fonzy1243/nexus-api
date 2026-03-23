use crate::{
    entity::{
        comments::{self, Entity as Comments},
        posts::{self, Entity as Posts},
        users::{self, Entity as Users},
    },
    error::{AppError, Result},
    state::AppState,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Params {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Serialize)]
pub struct PostSummary {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub author: String,
    pub community_id: Uuid,
    pub created_at: String,
    pub is_pinned: bool,
}

pub struct Query;

impl Query {
    pub async fn find_user_by_id(state: &AppState, id: Uuid) -> Result<users::Model> {
        Users::find_by_id(id)
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)
    }

    pub async fn get_user_posts(
        state: &AppState,
        user_id: Uuid,
        params: Params,
    ) -> Result<Vec<PostSummary>> {
        let page = params.page.unwrap_or(1).saturating_sub(1);
        let limit = Ord::min(params.limit.unwrap_or(20), 100u64);

        let posts = Posts::find()
            .filter(posts::Column::UserId.eq(user_id))
            .order_by_desc(posts::Column::CreatedAt)
            .find_also_related(Users)
            .offset(page * limit)
            .limit(limit)
            .all(&state.db)
            .await?;

        let result = posts
            .into_iter()
            .filter_map(|(post, user)| {
                let author = user?.username;
                Some(PostSummary {
                    id: post.id,
                    title: post.title,
                    body: post.body,
                    author,
                    community_id: post.community_id,
                    created_at: post.created_at.to_string(),
                    is_pinned: post.is_pinned,
                })
            })
            .collect();

        Ok(result)
    }

    pub async fn get_user_comments(
        state: &AppState,
        user_id: Uuid,
        params: Params,
    ) -> Result<Vec<comments::Model>> {
        let page = params.page.unwrap_or(1).saturating_sub(1);
        let limit = Ord::min(params.limit.unwrap_or(20), 100u64);

        let comments = Comments::find()
            .filter(comments::Column::UserId.eq(user_id))
            .order_by_desc(comments::Column::CreatedAt)
            .offset(page * limit)
            .limit(limit)
            .all(&state.db)
            .await?;

        Ok(comments)
    }
}
