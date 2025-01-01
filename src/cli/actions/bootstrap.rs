use std::{process::ExitCode, sync::Arc};

use async_trait::async_trait;
use sqlx::{Pool, Postgres};

use crate::{
    cli::{Action, ExitResult},
    models::{keywords::KeywordService, meals::MealService, restaurants::RestaurantService},
};

use super::{meals::MealsAction, restaurants::RestaurantAction, up::UpAction};

pub struct BootstrapAction {
    pub meal_action: Arc<MealsAction>,
    pub restaurant_action: Arc<RestaurantAction>,
    pub up_action: Arc<UpAction>,
}

impl BootstrapAction {
    pub fn new(
        pool: Arc<Pool<Postgres>>,
        meal_service: Arc<MealService>,
        restaurants_service: Arc<RestaurantService>,
        keyword_service: Arc<KeywordService>,
    ) -> Self {
        Self {
            meal_action: Arc::new(MealsAction::new(
                meal_service,
                restaurants_service.clone(),
                keyword_service.clone(),
            )),
            restaurant_action: Arc::new(RestaurantAction::new(
                restaurants_service,
                keyword_service,
            )),
            up_action: Arc::new(UpAction { pool: pool.clone() }),
        }
    }
}

#[async_trait]
impl Action for BootstrapAction {
    async fn execute(&self) -> Result<ExitResult, ExitResult> {
        self.up_action.execute().await?;
        self.restaurant_action.execute().await?;
        self.meal_action.execute().await?;
        Ok(ExitResult {
            exit_code: ExitCode::SUCCESS,
            message: "Environment bootstrapped successfully".to_string(),
        })
    }

    fn help(&self) -> &str {
        "calls every actions up -> restaurants -> meals, so in one action you can bootstrap a new database with all needed data"
    }
}
