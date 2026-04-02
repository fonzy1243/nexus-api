use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entity::{
        posts::{self, Entity as Posts},
        users::{self, Entity as Users},
    },
    error::Result,
    state::AppState,
};

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<u64>,
}

#[derive(Serialize)]
pub struct PostResult {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub author: String,
    pub author_id: Uuid,
    pub community_id: Uuid,
    pub is_pinned: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct UserResult {
    pub id: Uuid,
    pub username: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct SearchResults {
    pub posts: Vec<PostResult>,
    pub users: Vec<UserResult>,
}

pub struct Query;

impl Query {
    pub async fn search(state: &AppState, params: SearchParams) -> Result<SearchResults> {
        let q = params.q.trim().to_lowercase();
        let limit = Ord::min(params.limit.unwrap_or(20), 50);

        if q.is_empty() {
            return Ok(SearchResults {
                posts: vec![],
                users: vec![],
            });
        }

        // Search posts: match title or body (case-insensitive via LIKE)
        let like_pattern = format!("%{}%", q);

        let post_rows = Posts::find()
            .filter(
                sea_orm::Condition::any()
                    .add(posts::Column::Title.ilike(&like_pattern))
                    .add(posts::Column::Body.ilike(&like_pattern)),
            )
            .order_by_desc(posts::Column::CreatedAt)
            .find_also_related(Users)
            .limit(limit)
            .all(&state.db)
            .await?;

        let posts = post_rows
            .into_iter()
            .filter_map(|(post, user)| {
                let user = user?;
                Some(PostResult {
                    id: post.id,
                    title: post.title,
                    body: post.body,
                    author: user.username,
                    author_id: user.id,
                    community_id: post.community_id,
                    is_pinned: post.is_pinned,
                    created_at: post.created_at.to_string(),
                })
            })
            .collect();

        // Search users: match username
        let user_rows = Users::find()
            .filter(users::Column::Username.ilike(&like_pattern))
            .order_by_asc(users::Column::Username)
            .limit(limit)
            .all(&state.db)
            .await?;

        let users = user_rows
            .into_iter()
            .map(|u| UserResult {
                id: u.id,
                username: u.username,
                created_at: u.created_at.to_string(),
            })
            .collect();

        Ok(SearchResults { posts, users })
    }
}
