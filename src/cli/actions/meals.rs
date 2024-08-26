use std::{process::ExitCode, sync::Arc};

use async_trait::async_trait;
use futures::{io::Seek, FutureExt, TryFutureExt};
use scraper::{html::Select, Html, Selector};
use sqlx::PgPool;

use crate::{
    cli::{Action, ExitResult},
    models::{meals::Meal, restaurants},
};

pub struct MealsAction {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Action for MealsAction {
    async fn execute(&self) -> Result<ExitResult, ExitResult> {
        let restaurants_service = restaurants::RestaurantService {
            pool: self.pool.clone(),
        };

        let restaurants_url = restaurants_service
            .find_all()
            .await
            .map(|restaurants| {
                let mut restaurants_url: Vec<String> = Vec::new();
                for restaurant in restaurants.iter() {
                    restaurants_url.push(restaurant.url.to_string());
                }
                return restaurants_url;
            })
            .map_err(|e| {
                return ExitResult {
                    exit_code: ExitCode::from(2),
                    message: format!("can't find restaurants: {}", e),
                };
            });

        let tasks: Vec<_> = match restaurants_url {
            Ok(restaurants_url) => restaurants_url
                .into_iter()
                .map(|restaurant_url| {
                    tokio::spawn(async move {
                        match scrape_meals(restaurant_url).await {
                            Ok(meals) => meals,
                            Err(_) => Vec::new(),
                        }
                    })
                })
                .collect(),
            Err(exit_result) => return Err(exit_result),
        };

        for task in tasks {
            let meals = task.await.unwrap();
            for meal in meals.iter() {
                println!("{}: {} - {}", meal.day, meal.typemeal, meal.foodies);
            }
        }

        // Collect the results and handle any errors

        Ok(ExitResult {
            exit_code: ExitCode::from(1),
            message: "meals done".to_string(),
        })
    }
}

async fn scrape_meals(url: String) -> Result<Vec<Meal>, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&resp);
    let menu_selector = Selector::parse(".menu")?;
    let menu_element = document.select(&menu_selector);
    let date_selector = Selector::parse(".menu-date-title")?;
    let date_element = menu_element
        .into_iter()
        .next()
        .unwrap()
        .select(&date_selector);
    let date = date_element
        .into_iter()
        .next()
        .unwrap()
        .text()
        .collect::<String>();

    println!("Date : {}", date);

    Ok(vec![Meal {
        typemeal: "lunch".to_string(),
        foodies: "pasta".to_string(),
        day: date,
        idrestaurant: "1".to_string(),
    }])
}
