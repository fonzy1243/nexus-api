use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "bans")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    #[sea_orm(belongs_to, relation_enum = "BannedUser", from = "user_id", to = "id")]
    pub banned_user: HasOne<super::users::Entity>,
    pub banned_by: uuid::Uuid,
    #[sea_orm(belongs_to, relation_enum = "BannedBy", from = "banned_by", to = "id")]
    pub banner: HasOne<super::users::Entity>,
    pub community_id: Option<uuid::Uuid>,
    #[sea_orm(belongs_to, from = "community_id", to = "id")]
    pub community: HasOne<super::communities::Entity>,
    pub reason: String,
    pub created_at: ChronoDateTime,
    pub expires_at: ChronoDateTime,
}

impl ActiveModelBehavior for ActiveModel {}
