use std::{collections::HashMap, process::ExitCode, sync::Arc};

use async_trait::async_trait;
use scraper::{selectable::Selectable, Html, Selector};
use sqlx::PgPool;

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
        let mut restaurants = match scrape().await {
            Ok(restaurants) => restaurants,
            Err(_) => {
                return Err(ExitResult {
                    exit_code: ExitCode::from(2),
                    message: format!("scraping failed: {}", ""),
                });
            }
        };

        let mut restaurants_map = HashMap::new();

        let tasks = restaurants
            .iter_mut()
            .map(|restaurant| {
                let restaurant_url = restaurant.url.clone();
                let restaurant_name = restaurant.name.clone();
                restaurants_map.insert(restaurant_url.clone(), restaurant.clone());
                tokio::spawn(async move {
                    match scrape_coordinates(&restaurant_url).await {
                        Ok(gps) => gps,
                        Err(_) => {
                            println!("{}: no gps", restaurant_name);
                            RestaurantCoords {
                                restaurant: "".to_string(),
                                gps: "".to_string(),
                            }
                        }
                    }
                })
            })
            .collect::<Vec<_>>();

        let mut restaurants = Vec::new();

        for task in tasks {
            let restaurant_coords = task.await.unwrap();
            if restaurant_coords.gps.is_empty() {
                continue;
            }
            let restaurant = restaurants_map.get(restaurant_coords.restaurant.as_str());

            if restaurant.is_none() {
                continue;
            }

            let mut restaurant = restaurant.unwrap().clone();
            restaurant.gpscoord = Some(restaurant_coords.gps);

            restaurants.push(restaurant);
        }

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
    let resp = reqwest::get(url).await?;
    let text_resp = resp.text().await?;
    let document = Html::parse_document(&text_resp);
    let restaurant_selector = Selector::parse(".vc_restaurants ul li a")?;

    let elements = document.select(&restaurant_selector);

    let mut restaurants = Vec::new();

    for restaurant_element in elements {
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

        let restaurant_url = restaurant_element.value().attr("href");

        if restaurant_url.is_none() {
            continue;
        }

        let restaurant_url = restaurant_url.unwrap();

        let restaurant_name_selector = Selector::parse(".restaurant_title").unwrap();

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
    }

    Ok(restaurants)
}

#[derive(Debug)]
struct RestaurantCoords {
    restaurant: String,
    gps: String,
}

async fn scrape_coordinates(url: &str) -> Result<RestaurantCoords, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&resp);
    let map_selector = Selector::parse("#map")?;
    let map_element = document.select(&map_selector).next();
    if map_element.is_none() {
        return Err("no map element found".into());
    }
    let map_element = map_element.unwrap();
    let lat = map_element.value().attr("data-lat");
    if lat.is_none() {
        return Err("no lattitude found".into());
    }
    let lat = lat.unwrap();

    let long = map_element.value().attr("data-lon");

    if long.is_none() {
        return Err("no longitude found".into());
    }

    let long = long.unwrap();

    Ok(RestaurantCoords {
        restaurant: url.to_string(),
        gps: format!("point({},{})", lat, long),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scrape() {
        let restaurants = scrape().await.unwrap();
        assert!(restaurants.len() > 0);
    }

    #[tokio::test]
    async fn test_scrape_coordinates() {
        let gps =
            scrape_coordinates("https://www.crous-montpellier.fr/restaurant/brasserie-veyrassi-2/")
                .await;

        if gps.is_err() {
            println!("{:?}", gps);
        }

        assert!(gps.is_ok());
    }
}
