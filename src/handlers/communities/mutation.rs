use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use uuid::Uuid;

use super::query::Query;

use crate::{
    entity::{
        communities::{self, Entity as Communities},
        subscriptions::{self, Entity as Subscriptions, SubRole},
        users::UserRole,
    },
    error::{AppError, Result},
    extractors::AuthUser,
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

#[derive(Deserialize)]
pub struct UpdateModeratorInput {
    pub user_id: Uuid,
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

        subscriptions::ActiveModel {
            community_id: Set(id),
            subscriber_id: Set(user_id),
            role: Set(SubRole::Moderator),
            created_at: Set(chrono::Utc::now()),
        }
        .insert(&state.db)
        .await?;

        let _ = log(state, user_id, action::CREATE, target::COMMUNITY, id).await;

        Ok(community)
    }

    pub async fn update_community(
        state: &AppState,
        auth: &AuthUser,
        community_id: Uuid,
        input: UpdateCommunityInput,
    ) -> Result<communities::Model> {
        if auth.role != UserRole::Admin {
            if !Query::is_moderator(state, auth.id, community_id).await? {
                return Err(AppError::Unauthorized(
                    "You must be an admin or moderator to perform this action".into(),
                ));
            }
        }

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
            auth.id,
            action::UPDATE,
            target::COMMUNITY,
            community_id,
        )
        .await;

        Ok(updated)
    }

    pub async fn delete_community(
        state: &AppState,
        auth: &AuthUser,
        community_id: Uuid,
    ) -> Result<()> {
        if auth.role != UserRole::Admin {
            return Err(AppError::Unauthorized(
                "You must be an admin to perform this action".into(),
            ));
        }

        Communities::find_by_id(community_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        Communities::delete_by_id(community_id)
            .exec(&state.db)
            .await?;

        let _ = log(
            state,
            auth.id,
            action::DELETE,
            target::COMMUNITY,
            community_id,
        )
        .await;

        Ok(())
    }

    pub async fn make_moderator(
        state: &AppState,
        auth: &AuthUser,
        community_id: Uuid,
        input: UpdateModeratorInput,
    ) -> Result<()> {
        if auth.role != UserRole::Admin {
            if !Query::is_moderator(state, auth.id, community_id).await? {
                return Err(AppError::Unauthorized(
                    "You must be an admin or moderator to perform this action".into(),
                ));
            }
        }

        let sub = Subscriptions::find()
            .filter(subscriptions::Column::CommunityId.eq(community_id))
            .filter(subscriptions::Column::SubscriberId.eq(input.user_id))
            .one(&state.db)
            .await?
            .ok_or(AppError::BadRequest(
                "User is not a member of this community".into(),
            ))?;

        let mut active: subscriptions::ActiveModel = sub.into();
        active.role = Set(SubRole::Moderator);
        active.update(&state.db).await?;

        let _ = log(
            state,
            auth.id,
            action::UPDATE,
            target::COMMUNITY,
            community_id,
        )
        .await;

        Ok(())
    }

    pub async fn remove_moderator(
        state: &AppState,
        auth: &AuthUser,
        community_id: Uuid,
        input: UpdateModeratorInput,
    ) -> Result<()> {
        if auth.role != UserRole::Admin {
            if !Query::is_moderator(state, auth.id, community_id).await? {
                return Err(AppError::Unauthorized(
                    "You must be an admin or moderator to perform this action".into(),
                ));
            }
        }

        let sub = Subscriptions::find()
            .filter(subscriptions::Column::CommunityId.eq(community_id))
            .filter(subscriptions::Column::SubscriberId.eq(input.user_id))
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: subscriptions::ActiveModel = sub.into();
        active.role = Set(SubRole::Subscriber);
        active.update(&state.db).await?;

        let _ = log(
            state,
            auth.id,
            action::UPDATE,
            target::COMMUNITY,
            community_id,
        )
        .await;

        Ok(())
    }

    pub async fn join_community(state: &AppState, user_id: Uuid, community_id: Uuid) -> Result<()> {
        Communities::find_by_id(community_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let existing = Subscriptions::find()
            .filter(subscriptions::Column::CommunityId.eq(community_id))
            .filter(subscriptions::Column::SubscriberId.eq(user_id))
            .one(&state.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest(
                "Already a member of this community".into(),
            ));
        }

        subscriptions::ActiveModel {
            community_id: Set(community_id),
            subscriber_id: Set(user_id),
            role: Set(SubRole::Subscriber),
            created_at: Set(chrono::Utc::now()),
        }
        .insert(&state.db)
        .await?;

        let _ = log(
            state,
            user_id,
            action::CREATE,
            target::SUBSCRIPTION,
            community_id,
        )
        .await;

        Ok(())
    }

    pub async fn leave_community(
        state: &AppState,
        user_id: Uuid,
        community_id: Uuid,
    ) -> Result<()> {
        let sub = Subscriptions::find()
            .filter(subscriptions::Column::CommunityId.eq(community_id))
            .filter(subscriptions::Column::SubscriberId.eq(user_id))
            .one(&state.db)
            .await?
            .ok_or(AppError::BadRequest(
                "Not a member of this community".into(),
            ))?;

        Subscriptions::delete_many()
            .filter(subscriptions::Column::CommunityId.eq(community_id))
            .filter(subscriptions::Column::SubscriberId.eq(user_id))
            .exec(&state.db)
            .await?;

        let _ = log(
            state,
            user_id,
            action::DELETE,
            target::SUBSCRIPTION,
            community_id,
        );

        Ok(())
    }
}
