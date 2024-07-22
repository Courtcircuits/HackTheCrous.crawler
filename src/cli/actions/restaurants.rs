use std::{process::ExitCode, sync::Arc};

use async_trait::async_trait;
use sqlx::PgPool;

use crate::cli::{Action, ExitResult};

pub struct RestaurantAction {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Action for RestaurantAction {
    async fn execute(&self) -> Result<ExitResult, ExitResult> {
        Ok(ExitResult {
            exit_code: ExitCode::from(1),
            message: "restaurants in database".to_string(),
        })
    }
}
