use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: uuid::Uuid,
    pub is_pinned: bool,
    #[sea_orm(column_type = "Text")]
    pub body: String,
    #[sea_orm(nullable)]
    pub media_key: Option<String>,
    pub user_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: HasOne<super::users::Entity>,
    pub post_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "post_id", to = "id")]
    pub post: HasOne<super::posts::Entity>,
    #[sea_orm(nullable)]
    pub parent_id: Option<uuid::Uuid>,
    #[sea_orm(self_ref, relation_enum = "RepliesTo", from = "parent_id", to = "id")]
    pub parent: HasOne<Entity>,
    pub created_at: ChronoDateTime,
    pub edited_at: ChronoDateTime,
}

impl ActiveModelBehavior for ActiveModel {}
