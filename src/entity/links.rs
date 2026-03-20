use sea_orm::entity::prelude::*;

pub struct UserToFollowers;

impl Linked for UserToFollowers {
    type FromEntity = super::users::Entity;
    type ToEntity = super::users::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::follows::Relation::Following.def().rev(), // users -> follows
            super::follows::Relation::Follower.def(),        // follows -> users
        ]
    }
}

pub struct UserToFollowing;

impl Linked for UserToFollowing {
    type FromEntity = super::users::Entity;
    type ToEntity = super::users::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::follows::Relation::Follower.def().rev(), // users -> follows
            super::follows::Relation::Following.def(),      // follows -> users
        ]
    }
}

pub struct UserToBannedUsers;

impl Linked for UserToBannedUsers {
    type FromEntity = super::users::Entity;
    type ToEntity = super::users::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::bans::Relation::BannedBy.def().rev(), // users -> bans
            super::bans::Relation::BannedUser.def(),     // bans -> users
        ]
    }
}

pub struct UserToBanners;

impl Linked for UserToBanners {
    type FromEntity = super::users::Entity;
    type ToEntity = super::users::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::bans::Relation::BannedUser.def().rev(), // users -> bans
            super::bans::Relation::BannedBy.def(),         // bans -> users
        ]
    }
}
