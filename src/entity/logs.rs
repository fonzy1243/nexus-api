use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: uuid::Uuid,
    pub actor_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "actor_id", to = "id")]
    pub actor: HasOne<super::users::Entity>,
    pub action: String,
    pub target_type: String,
    pub target_id: uuid::Uuid,
    pub created_at: ChronoDateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}
