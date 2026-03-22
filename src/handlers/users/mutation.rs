use crate::{
    auth::create_token,
    entity::users::{self, Entity as Users},
    error::{AppError, Result},
    state::AppState,
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
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

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
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
        };

        user.insert(&state.db).await?;

        let token = create_token(id, &state.jwt_secret)?;
        Ok(AuthResponse {
            token,
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

        let token = create_token(user.id, &state.jwt_secret)?;
        Ok(AuthResponse {
            token,
            user_id: user.id,
            username: user.username,
        })
    }
}
