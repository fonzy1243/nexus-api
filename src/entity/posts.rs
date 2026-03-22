use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: uuid::Uuid,
    #[sea_orm(default_value = false)]
    pub is_pinned: bool,
    pub title: String,
    #[sea_orm(nullable)]
    pub media_key: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub body: String,
    pub user_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: HasOne<super::users::Entity>,
    pub community_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "community_id", to = "id")]
    pub community: HasOne<super::communities::Entity>,
    pub created_at: ChronoDateTime,
    pub edited_at: ChronoDateTime,
    #[sea_orm(has_many)]
    pub comments: HasMany<super::comments::Entity>,
    #[sea_orm(has_many)]
    pub cross_posts: HasMany<super::cross_posts::Entity>,
    #[sea_orm(has_many)]
    pub votes: HasMany<super::votes::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
