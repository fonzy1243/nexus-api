// use crate deps
use sea_orm::*;

use crate::{
    entity::logs::{self, Entity as Logs},
    error::Result,
    state::AppState,
};

pub struct Query;

impl Query {
    pub async fn get_all_logs(state: &AppState) -> Result<Vec<logs::Model>> {
        let logs = Logs::find()
            .order_by_desc(logs::Column::CreatedAt)
            .all(&state.db)
            .await?;

        Ok(logs)
    }
}
