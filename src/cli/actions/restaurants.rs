use std::{collections::HashMap, process::ExitCode, sync::Arc};

use async_trait::async_trait;
use scraper::{selectable::Selectable, Html, Selector};
use sqlx::{PgPool, Pool};

use crate::{
    cli::{Action, ExitResult},
    models::restaurants::{self, Restaurant},
};

pub struct RestaurantAction {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Action for RestaurantAction {
    async fn execute(&self) -> Result<ExitResult, ExitResult> {
        let restaurants = match scrape().await {
            Ok(restaurants) => restaurants,
            Err(err) => {
                return Err(ExitResult {
                    exit_code: ExitCode::from(2),
                    message: format!("scraping failed: {}", err),
                });
            }
        };

        let restaurant_service = restaurants::RestaurantService {
            pool: self.pool.clone(),
        };

        match restaurant_service.clear().await {
            Ok(_) => (),
            Err(err) => {
                return Err(ExitResult {
                    exit_code: ExitCode::from(2),
                    message: format!("clear failed: {}", err),
                });
            }
        }

        for restaurant in restaurants {
            match restaurant_service.create(restaurant).await {
                Ok(_) => (),
                Err(err) => {
                    return Err(ExitResult {
                        exit_code: ExitCode::from(2),
                        message: format!("insert failed: {}", err),
                    });
                }
            }
        }

        Ok(ExitResult {
            exit_code: ExitCode::from(1),
            message: "restaurants in database".to_string(),
        })
    }
}

async fn scrape() -> Result<Vec<Restaurant>, Box<dyn std::error::Error>> {
    let url = "https://www.crous-montpellier.fr/se-restaurer/ou-manger/";
    let resp = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&resp);
    let restaurant_selector = Selector::parse(".vc_restaurants ul li a")?;

    let mut restaurants = Vec::new();

    for restaurant_element in document.select(&restaurant_selector) {
        let city_selector = Selector::parse(".restaurant_area")?;

        if None == restaurant_element.select(&city_selector).next() {
            continue;
        }

        let city = restaurant_element
            .select(&city_selector)
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .join(" ");

        if !city.eq_ignore_ascii_case("Montpellier") && !city.eq_ignore_ascii_case("Sète") {
            continue;
        }

        let restaurant_url = restaurant_element.value().attr("href").unwrap();

        let restaurant_name_selector = Selector::parse(".restaurant_title")?;

        let restaurant_name = restaurant_element
            .select(&restaurant_name_selector)
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .join(" ");

        restaurants.push(Restaurant {
            idrestaurant: None,
            url: restaurant_url.to_string(),
            name: restaurant_name.to_string(),
            gpscoord: None,
            hours: None,
        });

        println!("scraped restaurant: {} in city: {}", restaurant_name, city);
    }

    Ok(restaurants)
}
