mod cli;
mod models;

use std::{env, process::ExitCode, sync::Arc};

use cli::{
    actions::{
        bootstrap::BootstrapAction, meals::MealsAction, restaurants::RestaurantAction, up::UpAction,
    },
    Cli, ExitResult,
};
use dotenv::dotenv;
use tracing::{error, info};

#[tokio::main]
async fn main() -> ExitCode {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let pg_database = get_env_variable("DATABASE_URL");
    let now = chrono::Utc::now();

    let pool = match pg_database {
        Ok(database) => {
            info!(database);
            let pool = sqlx::PgPool::connect(&database).await.unwrap();
            Arc::new(pool)
        }
        Err(err) => {
            error!("{}", err.message);
            return err.exit_code;
        }
    };

    let restaurant_service = Arc::new(models::restaurants::RestaurantService::new(pool.clone()));
    let keyword_service = Arc::new(models::keywords::KeywordService::new(pool.clone()));
    let meal_service = Arc::new(models::meals::MealService::new(pool.clone()));

    let restaurant_action =
        RestaurantAction::new(restaurant_service.clone(), keyword_service.clone());
    let meal_action = MealsAction::new(
        meal_service.clone(),
        restaurant_service.clone(),
        keyword_service.clone(),
    );

    let bootstrap_action = BootstrapAction::new(
        pool.clone(),
        meal_service.clone(),
        restaurant_service.clone(),
        keyword_service.clone(),
    );

    let result = &Cli::new()
        .subscribe_action("restaurants", restaurant_action)
        .subscribe_action("up", UpAction { pool: pool.clone() })
        .subscribe_action("meals", meal_action)
        .subscribe_action("bootstrap", bootstrap_action)
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
    env::var(key).map_err(|_| ExitResult {
        exit_code: ExitCode::from(2),
        message: format!("{} env variable not found", key),
    })
}
