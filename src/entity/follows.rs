use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "follows")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub following_id: uuid::Uuid,
    #[sea_orm(
        belongs_to,
        relation_enum = "Following",
        from = "following_id",
        to = "id"
    )]
    pub following: HasOne<super::users::Entity>,
    #[sea_orm(primary_key)]
    pub follower_id: uuid::Uuid,
    #[sea_orm(
        belongs_to,
        relation_enum = "Follower",
        from = "follower_id",
        to = "id"
    )]
    pub follower: HasOne<super::users::Entity>,
    pub created_at: ChronoDateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}
