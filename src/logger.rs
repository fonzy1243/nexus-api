use chrono::Utc;
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

use crate::{entity::logs, error::Result, state::AppState};

pub mod action {
    pub const LOGIN_SUCCESS: &str = "login_success";
    pub const LOGIN_FAILED: &str = "login_failed";
    pub const LOGOUT: &str = "logout";
    pub const REGISTER: &str = "register";
    pub const PASSWORD_CHANGE: &str = "password_change";
    pub const PASSWORD_RESET: &str = "password_reset";
    pub const USERNAME_CHANGE: &str = "username_change";
    pub const ACCOUNT_LOCKED: &str = "account_locked";
    pub const ACCESS_DENIED: &str = "access_denied";
    pub const VALIDATION_FAILED: &str = "validation_failed";
    pub const SECURITY_QUESTION_SET: &str = "security_question_set";
    pub const REFRESH: &str = "token_refresh";
    pub const CREATE: &str = "create";
    pub const UPDATE: &str = "update";
    pub const DELETE: &str = "delete";
}

pub mod target {
    pub const USER: &str = "user";
    pub const SESSION: &str = "session";
    pub const COMMUNITY: &str = "community";
    pub const POST: &str = "post";
    pub const COMMENT: &str = "comment";
    pub const SUBSCRIPTION: &str = "subscription";
}

pub async fn log(
    state: &AppState,
    actor_id: Uuid,
    action: &str,
    target_type: &str,
    target_id: Uuid,
) -> Result<()> {
    logs::ActiveModel {
        id: Set(Uuid::new_v4()),
        actor_id: Set(actor_id),
        action: Set(action.to_string()),
        target_type: Set(target_type.to_string()),
        target_id: Set(target_id),
        created_at: Set(Utc::now()),
    }
    .insert(&state.db)
    .await?;

    Ok(())
}
