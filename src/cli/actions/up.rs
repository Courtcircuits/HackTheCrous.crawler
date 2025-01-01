use std::{process::ExitCode, sync::Arc};

use async_trait::async_trait;
use sqlx::PgPool;

use crate::cli::{Action, ExitResult};

pub struct UpAction {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Action for UpAction {
    async fn execute(&self) -> Result<ExitResult, ExitResult> {
        sqlx::migrate!("./migrations/")
            .run(self.pool.as_ref())
            .await
            .map_err(|err| ExitResult {
                exit_code: ExitCode::from(2),
                message: format!("migration failed: {}", err),
            })?;

        Ok(ExitResult {
            exit_code: ExitCode::from(1),
            message: "migration done".to_string(),
        })
    }

    fn help(&self) -> &str {
        return "run the migrations"
    }
}
