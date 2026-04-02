use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entity::comments,
    entity::posts::Entity as Posts,
    error::{AppError, Result},
    handlers::communities::query::Query as CommunityQuery,
    logger::{action, log, target},
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreateCommentInput {
    pub body: String,
    pub media_key: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateCommentInput {
    pub body: Option<String>,
    pub media_key: Option<String>,
}

pub struct Mutation;

impl Mutation {
    pub async fn create_comment(
        state: &AppState,
        user_id: Uuid,
        post_id: Uuid,
        input: CreateCommentInput,
    ) -> Result<comments::Model> {
        if input.body.trim().is_empty() {
            return Err(AppError::BadRequest("Comment body cannot be empty".into()));
        }

        // If replying, verify the parent belongs to the same post
        if let Some(parent_id) = input.parent_id {
            let parent = comments::Entity::find_by_id(parent_id)
                .one(&state.db)
                .await?
                .ok_or(AppError::NotFound)?;

            if parent.post_id != post_id {
                return Err(AppError::BadRequest(
                    "Parent comment does not belong to this post".into(),
                ));
            }

            // Prevent deep nesting: only allow one level of replies
            if parent.parent_id.is_some() {
                return Err(AppError::BadRequest("Cannot reply to a reply".into()));
            }
        }

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let comment = comments::ActiveModel {
            id: Set(id),
            body: Set(input.body),
            media_key: Set(input.media_key),
            user_id: Set(user_id),
            post_id: Set(post_id),
            parent_id: Set(input.parent_id),
            is_pinned: Set(false),
            created_at: Set(now),
            edited_at: Set(now),
        }
        .insert(&state.db)
        .await?;

        let _ = log(state, user_id, action::CREATE, target::COMMENT, id).await;

        Ok(comment)
    }

    pub async fn update_comment(
        state: &AppState,
        user_id: Uuid,
        comment_id: Uuid,
        input: UpdateCommentInput,
        is_admin: bool,
    ) -> Result<comments::Model> {
        let comment = comments::Entity::find_by_id(comment_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let post = Posts::find_by_id(comment.post_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let is_moderator = CommunityQuery::is_moderator(state, user_id, post.community_id).await?;

        if comment.user_id != user_id && !is_admin && !is_moderator {
            let _ = log(
                state,
                user_id,
                action::ACCESS_DENIED,
                target::COMMENT,
                comment_id,
            )
            .await;
            return Err(AppError::Forbidden);
        }

        let mut active: comments::ActiveModel = comment.into();

        if let Some(body) = input.body {
            if body.trim().is_empty() {
                return Err(AppError::BadRequest("Comment body cannot be empty".into()));
            }
            active.body = Set(body);
        }
        if let Some(media_key) = input.media_key {
            active.media_key = Set(Some(media_key));
        }

        active.edited_at = Set(chrono::Utc::now());

        let updated = active.update(&state.db).await?;

        let _ = log(state, user_id, action::UPDATE, target::COMMENT, comment_id).await;

        Ok(updated)
    }

    pub async fn delete_comment(
        state: &AppState,
        user_id: Uuid,
        comment_id: Uuid,
        is_admin: bool,
    ) -> Result<()> {
        let comment = comments::Entity::find_by_id(comment_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let post = Posts::find_by_id(comment.post_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let is_moderator = CommunityQuery::is_moderator(state, user_id, post.community_id).await?;

        if comment.user_id != user_id && !is_admin && !is_moderator {
            let _ = log(
                state,
                user_id,
                action::ACCESS_DENIED,
                target::COMMENT,
                comment_id,
            )
            .await;
            return Err(AppError::Forbidden);
        }

        comments::Entity::delete_by_id(comment_id)
            .exec(&state.db)
            .await?;

        let _ = log(state, user_id, action::DELETE, target::COMMENT, comment_id).await;

        Ok(())
    }
}
