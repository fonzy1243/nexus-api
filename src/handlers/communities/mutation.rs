use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entity::communities,
    entity::communities::Entity as Communities,
    error::{AppError, Result},
    logger::{action, log, target},
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreateCommunityInput {
    pub name: String,
    pub logo: String,
}

#[derive(Deserialize)]
pub struct UpdateCommunityInput {
    pub name: Option<String>,
    pub logo: Option<String>,
}

pub struct Mutation;

impl Mutation {
    pub async fn create_community(
        state: &AppState,
        user_id: Uuid,
        input: CreateCommunityInput,
    ) -> Result<communities::Model> {
        let id = Uuid::new_v4();
        let community = communities::ActiveModel {
            id: Set(id),
            name: Set(input.name),
            logo: Set(input.logo),
            created_at: Set(chrono::Utc::now()),
        }
        .insert(&state.db)
        .await?;

        let _ = log(state, user_id, action::CREATE, target::COMMUNITY, id).await;

        Ok(community)
    }

    pub async fn update_community(
        state: &AppState,
        user_id: Uuid,
        community_id: Uuid,
        input: UpdateCommunityInput,
    ) -> Result<communities::Model> {
        let community = Communities::find_by_id(community_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: communities::ActiveModel = community.into();

        if let Some(name) = input.name {
            if name.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "Community name cannot be empty".into(),
                ));
            }
            active.name = Set(name);
        }
        if let Some(logo) = input.logo {
            active.logo = Set(logo);
        }

        let updated = active.update(&state.db).await?;

        let _ = log(
            state,
            user_id,
            action::UPDATE,
            target::COMMUNITY,
            community_id,
        )
        .await;

        Ok(updated)
    }

    pub async fn delete_community(
        state: &AppState,
        admin_id: Uuid,
        community_id: Uuid,
    ) -> Result<()> {
        Communities::find_by_id(community_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        Communities::delete_by_id(community_id)
            .exec(&state.db)
            .await?;

        let _ = log(
            state,
            admin_id,
            action::DELETE,
            target::COMMUNITY,
            community_id,
        )
        .await;

        Ok(())
    }
}
