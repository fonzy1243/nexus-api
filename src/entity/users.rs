use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: uuid::Uuid,
    pub username: String,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    pub security_question: Option<SecurityQuestion>,
    pub security_answer_hash: Option<String>,
    pub role: UserRole,
    pub created_at: ChronoDateTime,
    #[sea_orm(default_value = 0)]
    pub token_version: i32,
    pub failed_login_attempts: i32,
    pub locked_until: Option<ChronoDateTimeUtc>,
    pub last_login_at: Option<ChronoDateTimeUtc>,
    pub password_changed_at: ChronoDateTimeUtc,
    #[sea_orm(has_many)]
    pub posts: HasMany<super::posts::Entity>,
    #[sea_orm(has_many)]
    pub subscriptions: HasMany<super::subscriptions::Entity>,
    #[sea_orm(has_many)]
    pub cross_posts: HasMany<super::cross_posts::Entity>,
    #[sea_orm(has_many)]
    pub votes: HasMany<super::votes::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
pub enum UserRole {
    User,
    Admin,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
pub enum SecurityQuestion {
    FirstPet,          // "What was the name of your first pet?"
    ChildhoodNickname, // "What was your childhood nickname?"
    FirstCarModel,     // "What was the model of your first car?"
}

impl SecurityQuestion {
    pub fn as_text(&self) -> &'static str {
        match self {
            Self::FirstPet => "What was the name of your first pet?",
            Self::ChildhoodNickname => "What was your childhood nickname?",
            Self::FirstCarModel => "What was the model of your first car?",
        }
    }
}
