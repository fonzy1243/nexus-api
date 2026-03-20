use sea_orm::{entity::prelude::*, sqlx::types::uuid};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "subscriptions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub community_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "community_id", to = "id")]
    pub community: HasOne<super::communities::Entity>,
    #[sea_orm(primary_key)]
    pub subscriber_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "subscriber_id", to = "id")]
    pub subscriber: HasOne<super::users::Entity>,
    pub role: SubRole,
    pub created_at: ChronoDateTime,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
pub enum SubRole {
    Subscriber,
    Moderator,
}
