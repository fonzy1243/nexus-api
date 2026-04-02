use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::{
    auth::verify_token,
    entity::users::{self, UserRole},
    error::AppError,
    logger::{action, log, target},
    state::AppState,
};

pub struct AuthUser {
    pub id: Uuid,
    pub role: UserRole,
}

pub struct AdminUser {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::Unauthorized("Invalid authorization header".into()))?;

        let claims = verify_token(bearer.token(), &state.jwt_secret)?;

        let user = users::Entity::find_by_id(claims.sub)
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized("Account no longer exists".into()))?;

        if user.token_version != claims.version {
            let _ = log(state, user.id, action::ACCESS_DENIED, target::USER, user.id).await;
            return Err(AppError::Unauthorized("Session expired".into()));
        }

        Ok(AuthUser {
            id: claims.sub,
            role: user.role,
        })
    }
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let auth = AuthUser::from_request_parts(parts, state).await?;

        if auth.role != UserRole::Admin {
            let _ = log(state, auth.id, action::ACCESS_DENIED, target::USER, auth.id).await;
            return Err(AppError::Unauthorized(
                "You must be an admin to perform this action".into(),
            ));
        }

        Ok(AdminUser { id: auth.id })
    }
}
