use sea_orm::{sea_query::Expr, *};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entity::{
        posts::{self, Entity as Posts},
        users::{self, Entity as Users},
    },
    error::{AppError, Result},
    state::AppState,
};

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Serialize)]
pub struct UserResult {
    pub id: Uuid,
    pub username: String,
    pub created_at: String,
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
pub struct SearchResults {
    pub users: Vec<UserResult>,
    pub posts: Vec<PostResult>,
}

pub struct Query;

impl Query {
    pub async fn search(state: &AppState, params: SearchParams) -> Result<SearchResults> {
        let trimmed = params.q.trim().to_string();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest("Search query cannot be empty".into()));
        }

        let page = params.page.unwrap_or(1).saturating_sub(1);
        let limit = Ord::min(params.limit.unwrap_or(10), 50u64);
        let pattern = format!("%{}%", trimmed);

        // Users: ILIKE on username
        let user_rows = Users::find()
            .filter(Expr::cust_with_values(
                "\"users\".\"username\" ILIKE $1",
                [pattern.clone()],
            ))
            .order_by_asc(users::Column::Username)
            .offset(page * limit)
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

        // Posts: ILIKE on title OR body
        let post_rows = Posts::find()
            .filter(Expr::cust_with_values(
                "\"posts\".\"title\" ILIKE $1 OR \"posts\".\"body\" ILIKE $1",
                [pattern.clone()],
            ))
            .find_also_related(Users)
            .order_by_desc(posts::Column::CreatedAt)
            .offset(page * limit)
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

        Ok(SearchResults { users, posts })
    }
}
