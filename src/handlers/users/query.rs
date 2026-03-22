use crate::{
    entity::{users, users::Entity as Users},
    error::{AppError, Result},
    state::AppState,
};
use axum::extract::State;
use sea_orm::{sqlx::types::uuid, *};

pub struct Query;

impl Query {
    async fn find_user_by_id(state: State<AppState>, id: uuid::Uuid) -> Result<users::Model> {
        Users::find_by_id(id)
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound)
    }
}
