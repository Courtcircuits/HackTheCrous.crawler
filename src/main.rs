mod applications;
mod cli;
mod repositories;
mod telemetry;

use clap::Parser;
use std::{env, process::ExitCode, sync::Arc};
use telemetry::log::init_logger;

use cli::{
    actions::{
        bootstrap::BootstrapAction, meals::MealsAction, ping::PingAction,
        restaurants::RestaurantAction, schools::SchoolAction, up::UpAction,
    },
    Action, App, Cli, Command, ExitResult,
};
use dotenv::dotenv;
use tracing::{error, info, span, Level};

use crate::repositories::postgres::{
    keywords::KeywordService, meals::PostgresMealService, restaurants::PostgresRestaurantService,
    schools::PostgresSchoolService,
};

#[tokio::main]
async fn main() -> ExitCode {
    dotenv().ok();

    let args = App::parse();
    match get_env_variable("LOKI_ENDPOINT") {
        Ok(endpoint) => init_logger(Some(endpoint), &args.action).await,
        Err(_) => init_logger(None, &args.action).await,
    }

    let span = span!(Level::TRACE, "my_span");
    let _entre = span.enter();

    let now = chrono::Utc::now();

    if args.ping {
        match PingAction::new("https://www.crous-montpellier.fr/se-restaurer/ou-manger/")
            .execute()
            .await
        {
            Ok(res) => {
                info!("{}", res.message);
            }
            Err(res) => {
                error!("{}", res.message);
                return res.exit_code;
            }
        }
    }

    let pg_database = get_env_variable("DATABASE_URL");
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

    let restaurant_service = Arc::new(PostgresRestaurantService::new(pool.clone()));
    let keyword_service = Arc::new(KeywordService::new(pool.clone()));
    let meal_service = Arc::new(PostgresMealService::new(pool.clone()));

    let school_service = Arc::new(PostgresSchoolService::new(pool.clone()));

    let school_action = SchoolAction::new(school_service.clone());

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
        .subscribe_action(Command::Restaurants, restaurant_action)
        .subscribe_action(Command::Up, UpAction { pool: pool.clone() })
        .subscribe_action(Command::Meals, meal_action)
        .subscribe_action(Command::Bootstrap, bootstrap_action)
        .subscribe_action(Command::Schools, school_action)
        .execute(args)
        .await;

    match result {
        Ok(exit_result) => {
            info!("{}", exit_result.message);
            info!("took: {}", chrono::Utc::now().signed_duration_since(now));
            exit_result.exit_code
        }
        Err(exit_result) => {
            error!("{}", exit_result.message);
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
