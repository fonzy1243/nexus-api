use crate::{
    auth::{create_refresh_token, create_token, verify_refresh_token},
    entity::{
        password_history::{self, Entity as PasswordHistory},
        refresh_tokens::{self, Entity as RefreshTokens},
        users::{self, Entity as Users, SecurityQuestion, UserRole},
    },
    error::{AppError, Result},
    extractors::AuthUser,
    logger::{action, log, target},
    state::AppState,
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
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
    pub target_user_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub confirm_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct SetSecurityQuestionInput {
    pub question: SecurityQuestion,
    pub answer: String,
    pub current_password: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordInput {
    pub email: String,
    pub security_answer: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub username: String,
    pub last_login_at: Option<String>,
}

pub struct Mutation;

fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".into(),
        ));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(AppError::BadRequest(
            "Password must contain an uppercase letter".into(),
        ));
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err(AppError::BadRequest(
            "Password must contain a number".into(),
        ));
    }

    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(AppError::BadRequest(
            "Password must contain a special character".into(),
        ));
    }
    Ok(())
}

impl Mutation {
    pub async fn register(state: &AppState, input: RegisterInput) -> Result<AuthResponse> {
        validate_password(&input.password)?;

        let existing = Users::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&state.db)
            .await?;

        if existing.is_some() {
            let _ = log(
                state,
                Uuid::nil(),
                action::VALIDATION_FAILED,
                target::USER,
                Uuid::nil(),
            )
            .await;
            return Err(AppError::BadRequest("Account already exists.".into()));
        }

        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(input.password.as_bytes(), &salt)
            .map_err(|_| AppError::BadRequest("Error registering.".into()))?
            .to_string();

        let id = Uuid::new_v4();
        let active = users::ActiveModel {
            id: Set(id),
            username: Set(input.username.clone()),
            email: Set(input.email),
            password_hash: Set(hash),
            security_question: Set(None),
            security_answer_hash: Set(None),
            role: Set(users::UserRole::User),
            created_at: Set(chrono::Utc::now().naive_utc()),
            token_version: Set(0),
            failed_login_attempts: Set(0),
            last_login_at: Set(Option::Some(chrono::Utc::now())),
            locked_until: Set(Option::None),
            password_changed_at: Set(chrono::Utc::now()),
        };

        active.insert(&state.db).await?;

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

        let _ = log(state, id, action::REGISTER, target::USER, id).await;

        Ok(AuthResponse {
            access_token,
            refresh_token: raw_refresh,
            user_id: id,
            username: input.username,
            last_login_at: None,
        })
    }

    pub async fn login(state: &AppState, input: LoginInput) -> Result<AuthResponse> {
        let user = Users::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&state.db)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if let Some(locked_until) = user.locked_until {
            if locked_until > Utc::now() {
                let _ = log(state, user.id, action::LOGIN_FAILED, target::USER, user.id).await;
                return Err(AppError::BadRequest("Account temporarily locked".into()));
            }
        }

        let previous_login = user.last_login_at.map(|t| t.to_string());

        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| AppError::Unauthorized)?;

        let mut active: users::ActiveModel = user.clone().into();

        if Argon2::default()
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .is_err()
        {
            let attempts = user.failed_login_attempts + 1;
            active.failed_login_attempts = Set(attempts);

            if attempts >= 5 {
                active.locked_until = Set(Some(Utc::now() + chrono::TimeDelta::minutes(30)));
                let _ = log(
                    state,
                    user.id,
                    action::ACCOUNT_LOCKED,
                    target::USER,
                    user.id,
                )
                .await;
            }

            active.update(&state.db).await?;
            let _ = log(state, user.id, action::LOGIN_FAILED, target::USER, user.id).await;
            return Err(AppError::Unauthorized);
        }

        active.failed_login_attempts = Set(0);
        active.locked_until = Set(None);
        active.last_login_at = Set(Some(Utc::now()));
        active.update(&state.db).await?;

        let _ = log(state, user.id, action::LOGIN_SUCCESS, target::USER, user.id).await;

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
            last_login_at: previous_login,
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

        let _ = log(
            state,
            user.id,
            action::REFRESH,
            target::SESSION,
            token_row.id,
        );

        Ok(AuthResponse {
            access_token,
            refresh_token: raw_refresh,
            user_id: user.id,
            username: user.username,
            last_login_at: user.last_login_at.map(|t| t.to_string()),
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
                let token_id = token.id;
                RefreshTokens::delete_by_id(token.id)
                    .exec(&state.db)
                    .await?;
                let _ = log(state, user_id, action::LOGOUT, target::SESSION, token_id);
                break;
            }
        }

        Ok(())
    }

    pub async fn change_username(
        state: &AppState,
        auth: &AuthUser,
        input: ChangeUsernameInput,
    ) -> Result<users::Model> {
        let target_id = match input.target_user_id {
            Some(id) if auth.role == UserRole::Admin => id,
            Some(id) if id != auth.id => {
                let _ = log(state, auth.id, action::ACCESS_DENIED, target::USER, id).await;
                return Err(AppError::Unauthorized);
            }
            Some(id) => id,
            None => auth.id,
        };
        let taken = Users::find()
            .filter(users::Column::Username.eq(&input.username))
            .one(&state.db)
            .await?;

        if taken.is_some() {
            return Err(AppError::BadRequest("Username taken".into()));
        }

        let user = Users::find_by_id(auth.id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let old_token_ver = user.token_version;

        let mut active: users::ActiveModel = user.into();
        active.username = Set(input.username);
        active.token_version = Set(old_token_ver + 1);
        let updated = active.update(&state.db).await?;

        RefreshTokens::delete_many()
            .filter(refresh_tokens::Column::UserId.eq(auth.id))
            .exec(&state.db)
            .await?;

        let _ = log(
            state,
            auth.id,
            action::USERNAME_CHANGE,
            target::USER,
            target_id,
        )
        .await;

        Ok(updated)
    }

    pub async fn reset_password(state: &AppState, input: ResetPasswordInput) -> Result<()> {
        if input.new_password != input.confirm_password {
            return Err(AppError::BadRequest("Passwords do not match".into()));
        }

        validate_password(&input.new_password)?;

        let user = Users::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&state.db)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let answer_hash = user
            .security_answer_hash
            .as_ref()
            .ok_or(AppError::Unauthorized)?;

        let parsed = PasswordHash::new(answer_hash).map_err(|_| AppError::Unauthorized)?;

        let answer_normalized = input.security_answer.trim().to_lowercase();
        Argon2::default()
            .verify_password(answer_normalized.as_bytes(), &parsed)
            .map_err(|_| AppError::Unauthorized)?;

        let age = Utc::now() - user.password_changed_at;
        if age < chrono::TimeDelta::days(1) {
            return Err(AppError::BadRequest(
                "Password was changed too recently".into(),
            ));
        }

        let history = PasswordHistory::find()
            .filter(password_history::Column::UserId.eq(user.id))
            .order_by_desc(password_history::Column::CreatedAt)
            .all(&state.db)
            .await?;

        for old in &history {
            let parsed =
                PasswordHash::new(&old.password_hash).map_err(|_| AppError::Unauthorized)?;
            if Argon2::default()
                .verify_password(input.new_password.as_bytes(), &parsed)
                .is_ok()
            {
                return Err(AppError::BadRequest("Cannot reuse passwords".into()));
            }
        }

        let salt = SaltString::generate(&mut OsRng);
        let new_hash = Argon2::default()
            .hash_password(input.new_password.as_bytes(), &salt)
            .map_err(|_| AppError::BadRequest("Failed to hash password".into()))?
            .to_string();

        let mut active: users::ActiveModel = user.clone().into();
        active.password_hash = Set(new_hash);
        active.password_changed_at = Set(Utc::now());
        active.token_version = Set(user.token_version + 1);
        active.update(&state.db).await?;

        RefreshTokens::delete_many()
            .filter(refresh_tokens::Column::UserId.eq(user.id))
            .exec(&state.db)
            .await?;

        let _ = log(
            state,
            user.id,
            action::PASSWORD_RESET,
            target::USER,
            user.id,
        )
        .await;

        Ok(())
    }

    pub async fn change_password(
        state: &AppState,
        user_id: Uuid,
        input: ChangePasswordInput,
    ) -> Result<()> {
        validate_password(&input.new_password)?;

        if input.new_password != input.confirm_password {
            let _ = log(
                state,
                user_id,
                action::VALIDATION_FAILED,
                target::USER,
                user_id,
            )
            .await;
            return Err(AppError::BadRequest("Passwords do not match".into()));
        }

        if input.current_password == input.new_password {
            let _ = log(
                state,
                user_id,
                action::VALIDATION_FAILED,
                target::USER,
                user_id,
            )
            .await;
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

        let age = Utc::now() - user.password_changed_at;
        if age < chrono::TimeDelta::days(1) {
            let _ = log(
                state,
                user_id,
                action::VALIDATION_FAILED,
                target::USER,
                user_id,
            )
            .await;
            return Err(AppError::BadRequest(
                "Password must be at least 1 day old before changing".to_string(),
            ));
        }

        let history = PasswordHistory::find()
            .filter(password_history::Column::UserId.eq(user_id))
            .order_by_desc(password_history::Column::CreatedAt)
            .all(&state.db)
            .await?;

        for old in &history {
            let parsed =
                PasswordHash::new(&old.password_hash).map_err(|_| AppError::Unauthorized)?;
            if Argon2::default()
                .verify_password(input.new_password.as_bytes(), &parsed)
                .is_ok()
            {
                let _ = log(
                    state,
                    user_id,
                    action::VALIDATION_FAILED,
                    target::USER,
                    user_id,
                )
                .await;
                return Err(AppError::BadRequest("Cannot reuse passwords".into()));
            }
        }

        let salt = SaltString::generate(&mut OsRng);
        let new_hash = Argon2::default()
            .hash_password(input.new_password.as_bytes(), &salt)
            .map_err(|_| AppError::BadRequest("Failed to change password".into()))?
            .to_string();

        let old_token_ver = user.token_version;

        let mut active: users::ActiveModel = user.clone().into();
        active.password_hash = Set(new_hash);
        active.token_version = Set(old_token_ver + 1);
        active.update(&state.db).await?;

        RefreshTokens::delete_many()
            .filter(refresh_tokens::Column::UserId.eq(user_id))
            .exec(&state.db)
            .await?;

        let _ = log(
            state,
            user.id,
            action::PASSWORD_CHANGE,
            target::USER,
            user.id,
        )
        .await;

        Ok(())
    }

    pub async fn get_security_question(state: &AppState, email: String) -> Result<String> {
        let user = Users::find()
            .filter(users::Column::Email.eq(&email))
            .one(&state.db)
            .await?
            .ok_or(AppError::Unauthorized)?;

        Ok(user
            .security_question
            .ok_or(AppError::BadRequest("No security question set".into()))?
            .as_text()
            .to_string())
    }

    pub async fn set_security_question(
        state: &AppState,
        user_id: Uuid,
        input: SetSecurityQuestionInput,
    ) -> Result<()> {
        let user = Users::find_by_id(user_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| AppError::Unauthorized)?;
        Argon2::default()
            .verify_password(input.current_password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized)?;

        let answer_normalized = input.answer.trim().to_lowercase();
        let salt = SaltString::generate(&mut OsRng);
        let answer_hash = Argon2::default()
            .hash_password(answer_normalized.as_bytes(), &salt)
            .map_err(|_| AppError::BadRequest("Failed to hash answer".into()))?
            .to_string();

        let mut active: users::ActiveModel = user.into();
        active.security_question = Set(Some(input.question));
        active.security_answer_hash = Set(Some(answer_hash));
        active.update(&state.db).await?;

        Ok(())
    }
}
