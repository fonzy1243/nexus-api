use sea_orm::{entity::prelude::*, sqlx::types::uuid};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "communities")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: uuid::Uuid,
    pub name: String,
    pub logo: String,
    pub created_at: ChronoDateTime,
    #[sea_orm(has_many)]
    pub posts: HasMany<super::posts::Entity>,
    #[sea_orm(has_many)]
    pub cross_posts: HasMany<super::cross_posts::Entity>,
    #[sea_orm(has_many)]
    pub subscribers: HasMany<super::subscriptions::Entity>,
    #[sea_orm(has_many)]
    pub bans: HasMany<super::bans::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
