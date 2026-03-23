use crate::{
    auth::{create_refresh_token, create_token, verify_refresh_token},
    entity::{
        refresh_tokens::{self, Entity as RefreshTokens},
        users::{self, Entity as Users},
    },
    error::{AppError, Result},
    state::AppState,
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshInput {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct ChangeUsernameInput {
    pub username: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub confirm_password: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub username: String,
}

pub struct Mutation;

impl Mutation {
    pub async fn register(state: &AppState, input: RegisterInput) -> Result<AuthResponse> {
        let existing = Users::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&state.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("Account already exists.".into()));
        }

        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(input.password.as_bytes(), &salt)
            .map_err(|_| AppError::BadRequest("Error registering.".into()))?
            .to_string();

        let id = Uuid::new_v4();
        let user = users::ActiveModel {
            id: Set(id),
            username: Set(input.username.clone()),
            email: Set(input.email),
            password_hash: Set(hash),
            role: Set(users::UserRole::User),
            created_at: Set(chrono::Utc::now().naive_utc()),
            token_version: Set(0),
        };

        user.insert(&state.db).await?;

        let access_token = create_token(id, &state.jwt_secret, 0)?;
        let (raw_refresh, hashed_refresh) = create_refresh_token()?;

        refresh_tokens::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(id),
            token_hash: Set(hashed_refresh),
            expires_at: Set(Utc::now() + Duration::days(30)),
            created_at: Set(Utc::now()),
        }
        .insert(&state.db)
        .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token: raw_refresh,
            user_id: id,
            username: input.username,
        })
    }

    pub async fn login(state: &AppState, input: LoginInput) -> Result<AuthResponse> {
        let user = Users::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&state.db)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| AppError::Unauthorized)?;

        Argon2::default()
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized)?;

        let access_token = create_token(user.id, &state.jwt_secret, user.token_version)?;
        let (raw_refresh, hashed_refresh) = create_refresh_token()?;

        refresh_tokens::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            token_hash: Set(hashed_refresh),
            expires_at: Set(Utc::now() + Duration::days(30)),
            created_at: Set(Utc::now()),
        }
        .insert(&state.db)
        .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token: raw_refresh,
            user_id: user.id,
            username: user.username,
        })
    }

    pub async fn refresh(state: &AppState, input: RefreshInput) -> Result<AuthResponse> {
        // Find matching refresh token
        let tokens = RefreshTokens::find()
            .filter(refresh_tokens::Column::ExpiresAt.gt(Utc::now()))
            .find_also_related(Users)
            .all(&state.db)
            .await?;

        let (token_row, user) = tokens
            .into_iter()
            .find(|(t, _)| verify_refresh_token(&input.refresh_token, &t.token_hash).is_ok())
            .ok_or(AppError::Unauthorized)?;

        let user = user.ok_or(AppError::Unauthorized)?;

        RefreshTokens::delete_by_id(token_row.id)
            .exec(&state.db)
            .await?;

        let access_token = create_token(user.id, &state.jwt_secret, user.token_version)?;
        let (raw_refresh, hashed_refresh) = create_refresh_token()?;

        refresh_tokens::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            token_hash: Set(hashed_refresh),
            expires_at: Set(Utc::now() + Duration::days(30)),
            created_at: Set(Utc::now()),
        }
        .insert(&state.db)
        .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token: raw_refresh,
            user_id: user.id,
            username: user.username,
        })
    }

    // Delete all refresh tokens
    pub async fn logout(state: &AppState, user_id: Uuid, input: RefreshInput) -> Result<()> {
        let tokens = RefreshTokens::find()
            .filter(refresh_tokens::Column::UserId.eq(user_id))
            .all(&state.db)
            .await?;

        for token in tokens {
            if verify_refresh_token(&input.refresh_token, &token.token_hash).is_ok() {
                RefreshTokens::delete_by_id(token.id)
                    .exec(&state.db)
                    .await?;
                break;
            }
        }

        Ok(())
    }

    pub async fn change_username(
        state: &AppState,
        user_id: Uuid,
        input: ChangeUsernameInput,
    ) -> Result<users::Model> {
        let taken = Users::find()
            .filter(users::Column::Username.eq(&input.username))
            .one(&state.db)
            .await?;

        if taken.is_some() {
            return Err(AppError::BadRequest("Username taken".into()));
        }

        let user = Users::find_by_id(user_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let old_token_ver = user.token_version;

        let mut active: users::ActiveModel = user.into();
        active.username = Set(input.username);
        active.token_version = Set(old_token_ver + 1);
        let updated = active.update(&state.db).await?;

        RefreshTokens::delete_many()
            .filter(refresh_tokens::Column::UserId.eq(user_id))
            .exec(&state.db)
            .await?;

        Ok(updated)
    }

    pub async fn change_password(
        state: &AppState,
        user_id: Uuid,
        input: ChangePasswordInput,
    ) -> Result<()> {
        if input.new_password != input.confirm_password {
            return Err(AppError::BadRequest("Passwords do not match".into()));
        }

        if input.current_password == input.new_password {
            return Err(AppError::BadRequest(
                "New password must differ from current password".into(),
            ));
        }

        let user = Users::find_by_id(user_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| AppError::Unauthorized)?;

        Argon2::default()
            .verify_password(input.current_password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized)?;

        let salt = SaltString::generate(&mut OsRng);
        let new_hash = Argon2::default()
            .hash_password(input.new_password.as_bytes(), &salt)
            .map_err(|_| AppError::BadRequest("Failed to change password".into()))?
            .to_string();

        let old_token_ver = user.token_version;

        let mut active: users::ActiveModel = user.into();
        active.password_hash = Set(new_hash);
        active.token_version = Set(old_token_ver + 1);
        active.update(&state.db).await?;

        RefreshTokens::delete_many()
            .filter(refresh_tokens::Column::UserId.eq(user_id))
            .exec(&state.db)
            .await?;

        Ok(())
    }
}
