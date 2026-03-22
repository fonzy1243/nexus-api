use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "votes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: HasOne<super::users::Entity>,
    pub post_id: Option<uuid::Uuid>,
    #[sea_orm(belongs_to, from = "post_id", to = "id")]
    pub post: HasOne<super::posts::Entity>,
    pub comment_id: Option<uuid::Uuid>,
    #[sea_orm(belongs_to, from = "comment_id", to = "id")]
    pub comment: HasOne<super::comments::Entity>,
    pub vote_type: i8,
    pub created_at: ChronoDateTime,
}

impl ActiveModelBehavior for ActiveModel {}
