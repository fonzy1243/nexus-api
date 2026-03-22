use sea_orm::entity::prelude::*;

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
    pub role: UserRole,
    pub created_at: ChronoDateTime,
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

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
pub enum UserRole {
    User,
    Admin,
}
