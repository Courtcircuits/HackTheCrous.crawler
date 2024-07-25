mod cli;
mod models;

use std::{env, process::ExitCode, sync::Arc};

use cli::{
    actions::{restaurants::RestaurantAction, up::UpAction},
    Cli, ExitResult,
};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> ExitCode {
    dotenv().ok();
    let pg_database = get_env_variable("DATABASE_URL");
    let now = chrono::Utc::now();

    let pool = match pg_database {
        Ok(database) => {
            let pool = sqlx::PgPool::connect(&database).await.unwrap();
            Arc::new(pool)
        }
        Err(err) => {
            println!("{}", err.message);
            return err.exit_code;
        }
    };

    let result = &Cli::new()
        .subscribe_action("restaurants", RestaurantAction { pool: pool.clone() })
        .subscribe_action("up", UpAction { pool: pool.clone() })
        .execute()
        .await;

    match result {
        Ok(exit_result) => {
            println!("{}", exit_result.message);
            println!("took: {}", chrono::Utc::now().signed_duration_since(now));
            exit_result.exit_code
        }
        Err(exit_result) => {
            println!("{}", exit_result.message);
            exit_result.exit_code
        }
    }
}

fn get_env_variable(key: &str) -> Result<String, ExitResult> {
    env::var(&key).map_err(|_| ExitResult {
        exit_code: ExitCode::from(2),
        message: format!("{} env variable not found", key),
    })
}
