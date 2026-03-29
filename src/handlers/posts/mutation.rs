use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entity::posts,
    entity::posts::Entity as Posts,
    error::{AppError, Result},
    logger::{action, log, target},
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreatePostInput {
    pub title: String,
    pub body: String,
    pub media_key: Option<String>,
    pub community_id: Uuid,
}

#[derive(Deserialize)]
pub struct UpdatePostInput {
    pub title: String,
    pub body: String,
    pub media_key: Option<String>,
}

pub struct Mutation;

impl Mutation {
    pub async fn create_post(
        state: &AppState,
        user_id: Uuid,
        input: CreatePostInput,
    ) -> Result<posts::Model> {
        if input.title.trim().is_empty() {
            return Err(AppError::BadRequest("Title cannot be empty".into()));
        }

        if input.body.trim().is_empty() {
            return Err(AppError::BadRequest("Body cannot be empty".into()));
        }

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let post = posts::ActiveModel {
            id: Set(id),
            title: Set(input.title),
            body: Set(input.body),
            media_key: Set(input.media_key),
            user_id: Set(user_id),
            community_id: Set(input.community_id),
            is_pinned: Set(false),
            created_at: Set(now),
            edited_at: Set(now),
        }
        .insert(&state.db)
        .await?;

        let _ = log(state, user_id, action::CREATE, target::USER, id).await;

        Ok(post)
    }

    pub async fn update_post(
        state: &AppState,
        user_id: Uuid,
        post_id: Uuid,
        input: UpdatePostInput,
        is_admin: bool,
    ) -> Result<posts::Model> {
        let post = Posts::find_by_id(post_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        if post.user_id != user_id && !is_admin {
            let _ = log(state, user_id, action::ACCESS_DENIED, target::POST, post_id).await;
            return Err(AppError::Forbidden);
        }

        let mut active: posts::ActiveModel = post.into();

        let title = input.title;
        if title.trim().is_empty() {
            return Err(AppError::BadRequest("Title cannot be empty".into()));
        }
        active.title = Set(title);

        let body = input.body;
        if body.trim().is_empty() {
            return Err(AppError::BadRequest("Body cannot be empty".into()));
        }
        active.body = Set(body);

        if let Some(media_key) = input.media_key {
            active.media_key = Set(Some(media_key));
        }

        active.edited_at = Set(chrono::Utc::now());

        let updated = active.update(&state.db).await?;

        let _ = log(state, user_id, action::UPDATE, target::POST, post_id).await;

        Ok(updated)
    }

    pub async fn delete_post(
        state: &AppState,
        user_id: Uuid,
        post_id: Uuid,
        is_admin: bool,
    ) -> Result<()> {
        let post = Posts::find_by_id(post_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        if post.user_id != user_id && !is_admin {
            let _ = log(state, user_id, action::ACCESS_DENIED, target::POST, post_id).await;
            return Err(AppError::Forbidden);
        }

        Posts::delete_by_id(post_id).exec(&state.db).await?;

        let _ = log(state, user_id, action::DELETE, target::POST, post_id).await;

        Ok(())
    }
}
