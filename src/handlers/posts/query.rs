// use crate deps
use sea_orm::{
    sea_query::SimpleExpr,
    sea_query::{Expr, Order},
    *,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entity::{
        comments::{self, Entity as Comments},
        posts::{self, Entity as Posts},
        users::{self, Entity as Users},
    },
    error::{AppError, Result},
    state::AppState,
};

#[derive(Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[default]
    CreatedAt,
    VoteCount,
}

#[derive(Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

#[derive(Deserialize)]
pub struct ListParams {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub sort_by: Option<SortBy>,
    pub sort_order: Option<SortOrder>,
}

impl ListParams {
    fn offset_and_limit(&self) -> (u64, u64) {
        let page = self.page.unwrap_or(1).saturating_sub(1);
        let limit = Ord::min(self.limit.unwrap_or(20), 100);

        (page * limit, limit)
    }
}

#[derive(Clone, Serialize)]
pub struct PostSummary {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub media_key: Option<String>,
    pub author: String,
    pub author_id: Uuid,
    pub community_id: Uuid,
    pub is_pinned: bool,
    pub created_at: String,
}

#[derive(Clone, Serialize)]
pub struct CommentSummary {
    pub id: Uuid,
    pub body: String,
    pub media_key: Option<String>,
    pub author: String,
    pub author_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub is_pinned: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct CommentWithReplies {
    #[serde(flatten)]
    pub comment: CommentSummary,
    pub replies: Vec<CommentSummary>,
}

fn post_vote_expr() -> SimpleExpr {
    Expr::cust("(SELECT COALESCE(SUM(vote_type), 0) FROM votes WHERE post_id = posts.id)")
}

fn comment_vote_expr() -> SimpleExpr {
    Expr::cust("(SELECT COALESCE(SUM(vote_type), 0) FROM votes WHERE comment_id = comments.id)")
}

fn map_post(post: posts::Model, user: Option<users::Model>) -> Option<PostSummary> {
    let user = user?;
    Some(PostSummary {
        id: post.id,
        title: post.title,
        body: post.body,
        media_key: post.media_key,
        author: user.username,
        author_id: user.id,
        community_id: post.community_id,
        is_pinned: post.is_pinned,
        created_at: post.created_at.to_string(),
    })
}

fn map_comment(comment: comments::Model, user: Option<users::Model>) -> Option<CommentSummary> {
    let user = user?;
    Some(CommentSummary {
        id: comment.id,
        body: comment.body,
        media_key: comment.media_key,
        author: user.username,
        author_id: user.id,
        parent_id: comment.parent_id,
        is_pinned: comment.is_pinned,
        created_at: comment.created_at.to_string(),
    })
}

pub struct Query;

impl Query {
    pub async fn fetch_posts(
        base: Select<Posts>,
        state: &AppState,
        params: ListParams,
    ) -> Result<Vec<PostSummary>> {
        let (offset, limit) = params.offset_and_limit();
        let sort_by = params.sort_by.unwrap_or_default();
        let sort_order = params.sort_order.unwrap_or_default();

        let select = base.find_also_related(Users);

        let select = match (sort_by, sort_order) {
            (SortBy::CreatedAt, SortOrder::Asc) => select.order_by_asc(posts::Column::CreatedAt),
            (SortBy::CreatedAt, SortOrder::Desc) => select.order_by_desc(posts::Column::CreatedAt),
            (SortBy::VoteCount, SortOrder::Asc) => select.order_by(post_vote_expr(), Order::Asc),
            (SortBy::VoteCount, SortOrder::Desc) => select.order_by(post_vote_expr(), Order::Desc),
        };

        let rows = select.offset(offset).limit(limit).all(&state.db).await?;

        Ok(rows
            .into_iter()
            .filter_map(|(p, u)| map_post(p, u))
            .collect())
    }

    pub async fn get_all_posts(state: &AppState, params: ListParams) -> Result<Vec<PostSummary>> {
        Self::fetch_posts(Posts::find(), state, params).await
    }

    pub async fn get_post_by_id(state: &AppState, post_id: Uuid) -> Result<PostSummary> {
        Posts::find_by_id(post_id)
            .find_also_related(Users)
            .one(&state.db)
            .await?
            .and_then(|(p, u)| map_post(p, u))
            .ok_or(AppError::NotFound)
    }

    pub async fn get_community_posts(
        state: &AppState,
        community_id: Uuid,
        params: ListParams,
    ) -> Result<Vec<PostSummary>> {
        let base = Posts::find().filter(posts::Column::CommunityId.eq(community_id));
        Self::fetch_posts(base, state, params).await
    }

    pub async fn get_post_comments(
        state: &AppState,
        post_id: Uuid,
        params: ListParams,
    ) -> Result<Vec<CommentWithReplies>> {
        let (offset, limit) = params.offset_and_limit();
        let sort_by = params.sort_by.unwrap_or_default();
        let sort_order = params.sort_order.unwrap_or_default();

        let top_level_select = Comments::find()
            .filter(comments::Column::PostId.eq(post_id))
            .filter(comments::Column::ParentId.is_null())
            .find_also_related(Users);

        let top_level_select = match (sort_by, sort_order) {
            (SortBy::CreatedAt, SortOrder::Asc) => {
                top_level_select.order_by_asc(comments::Column::CreatedAt)
            }
            (SortBy::CreatedAt, SortOrder::Desc) => {
                top_level_select.order_by_asc(comments::Column::CreatedAt)
            }
            (SortBy::VoteCount, SortOrder::Asc) => {
                top_level_select.order_by(comment_vote_expr(), Order::Asc)
            }
            (SortBy::VoteCount, SortOrder::Desc) => {
                top_level_select.order_by(comment_vote_expr(), Order::Desc)
            }
        };

        let top_level_rows = top_level_select
            .offset(offset)
            .limit(limit)
            .all(&state.db)
            .await?;

        let top_level: Vec<CommentSummary> = top_level_rows
            .into_iter()
            .filter_map(|(c, u)| map_comment(c, u))
            .collect();

        if top_level.is_empty() {
            return Ok(vec![]);
        }

        let parent_ids: Vec<Uuid> = top_level.iter().map(|c| c.id).collect();

        let reply_rows = Comments::find()
            .filter(comments::Column::PostId.eq(post_id))
            .filter(comments::Column::ParentId.is_in(parent_ids))
            .find_also_related(Users)
            .order_by_asc(comments::Column::CreatedAt)
            .all(&state.db)
            .await?;

        let replies: Vec<CommentSummary> = reply_rows
            .into_iter()
            .filter_map(|(c, u)| map_comment(c, u))
            .collect();

        let result = top_level
            .into_iter()
            .map(|comment| {
                let comment_id = comment.id;
                let my_replies = replies
                    .iter()
                    .filter(|r| r.parent_id == Some(comment_id))
                    .cloned()
                    .collect();
                CommentWithReplies {
                    comment,
                    replies: my_replies,
                }
            })
            .collect();

        Ok(result)
    }
}
