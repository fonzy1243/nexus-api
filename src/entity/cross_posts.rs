use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "cross_posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub post_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "post_id", to = "id")]
    pub post: HasOne<super::posts::Entity>,
    #[sea_orm(primary_key)]
    pub community_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "community_id", to = "id")]
    pub community: HasOne<super::communities::Entity>,
    #[sea_orm(primary_key)]
    pub user_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: HasOne<super::users::Entity>,
    pub created_at: ChronoDateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}
