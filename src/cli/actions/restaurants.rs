use std::{collections::HashMap, error::Error, process::ExitCode, sync::Arc};

use async_trait::async_trait;
use regex::Regex;
use scraper::{selectable::Selectable, Html, Selector};
use tracing::info;

use crate::{
    cli::{Action, ExitResult},
    models::{
        keywords::{Category, KeywordService},
        restaurants::{Restaurant, RestaurantService},
    },
};

pub struct RestaurantAction {
    pub restaurant_service: Arc<RestaurantService>,
    pub keyword_service: Arc<KeywordService>,
}

pub struct RestaurantDetails {
    pub restaurant: String,
    pub gps: String,
    pub hours: String,
}

impl RestaurantAction {
    pub fn new(
        restaurant_service: Arc<RestaurantService>,
        keyword_service: Arc<KeywordService>,
    ) -> Self {
        Self {
            restaurant_service,
            keyword_service,
        }
    }
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
                    let coordinates = match scrape_coordinates(&restaurant_url).await {
                        Ok(gps) => gps,
                        Err(_) => {
                            println!("{}: no gps", restaurant_name);
                            RestaurantCoords {
                                restaurant: "".to_string(),
                                gps: "".to_string(),
                            }
                        }
                    };
                    let hours = match scrape_hours(&restaurant_url).await {
                        Ok(hours) => hours,
                        Err(_) => {
                            println!("{}: no hours", restaurant_name);
                            "".to_string()
                        }
                    };
                    RestaurantDetails {
                        restaurant: coordinates.restaurant,
                        gps: coordinates.gps,
                        hours,
                    }
                })
            })
            .collect::<Vec<_>>();

        let mut restaurants = Vec::new();

        for task in tasks {
            let restaurant_details = task.await.unwrap();

            println!(
                "[Restaurant ${}] hours -> {}",
                restaurant_details.restaurant, restaurant_details.hours
            );

            if restaurant_details.gps.is_empty() {
                continue;
            }
            let restaurant = restaurants_map.get(restaurant_details.restaurant.as_str());

            if restaurant.is_none() {
                continue;
            }

            let mut restaurant = restaurant.unwrap().clone();
            restaurant.gpscoord = Some(restaurant_details.gps);

            restaurants.push(restaurant);
        }

        match self.restaurant_service.clear().await {
            Ok(_) => (),
            Err(err) => {
                return Err(ExitResult {
                    exit_code: ExitCode::from(2),
                    message: format!("clear failed: {}", err),
                });
            }
        }

        for restaurant in restaurants {
            match self.restaurant_service.create(restaurant).await {
                Ok(restaurant) => {
                    for word in restaurant.name.split_whitespace() {
                        self.keyword_service
                            .create(
                                word.to_string(),
                                i64::from(restaurant.idrestaurant.unwrap()),
                                Category::Restaurant,
                            )
                            .await
                            .map_err(|err| {
                                return ExitResult {
                                    exit_code: ExitCode::from(2),
                                    message: format!("keyword insertion failed: {}", err),
                                };
                            })?;
                    }
                }
                Err(err) => {
                    return Err(ExitResult {
                        exit_code: ExitCode::from(2),
                        message: format!("restaurant insertion failed: {}", err),
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

async fn scrape_hours(url: &str) -> Result<String, Box<dyn Error>> {
    let resp = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&resp);
    let hours_selector = Selector::parse(".info p")?;
    let hours = document.select(&hours_selector).next();
    if hours.is_none() {
        return Err("no hours found".into());
    }
    let hours = hours.unwrap().text().collect::<Vec<_>>().join(" ");
    if url == "https://www.crous-montpellier.fr/restaurant/resto-u-triolet/" {
        return Ok(parse_hours("du lundi au vendredi de 11h30 à 13h30."));
    }
    Ok(parse_hours(hours.as_str()))
}

fn parse_hours(raw_hour: &str) -> String {
    let raw_hour = raw_hour.to_lowercase();
    let re = Regex::new(r"du lundi au vendredi de |du lundi au jeudi de ").unwrap();
    let hours = re
        .split(raw_hour.as_str())
        .collect::<Vec<_>>()
        .last()
        .unwrap()
        .to_string()
        .replace(".", "")
        .split(" à ")
        .collect::<Vec<_>>()
        .iter()
        .map(|hour| match hour.split("h").collect::<Vec<_>>() {
            hour if hour.len() == 2 => {
                if hour[1].is_empty() {
                    vec![hour[0], "00"]
                } else {
                    hour
                }
            }
            hour if hour.len() == 1 => vec![hour[0], "00"],
            _ => vec!["00", "00"],
        })
        .map(|hour| format!("{}:{}", hour[0], hour[1]))
        .collect::<Vec<_>>();

    hours.join(" - ")
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