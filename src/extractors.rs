use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::{auth::verify_token, entity::users, error::AppError, state::AppState};

pub struct AuthUser {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::Unauthorized)?;

        let claims = verify_token(bearer.token(), &state.jwt_secret)?;

        let user = users::Entity::find_by_id(claims.sub)
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        if user.token_version != claims.version {
            return Err(AppError::Unauthorized);
        }

        Ok(AuthUser { id: claims.sub })
    }
}
