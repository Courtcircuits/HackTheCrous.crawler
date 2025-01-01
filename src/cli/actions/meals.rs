use std::{process::ExitCode, sync::Arc};

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    cli::{Action, ExitResult},
    models::{
        keywords::{Category, KeywordService},
        meals::{Meal, MealService},
        restaurants::{Restaurant, RestaurantService},
    },
};

pub struct MealsAction {
    pub meal_service: Arc<MealService>,
    pub restaurants_service: Arc<RestaurantService>,
    pub keyword_service: Arc<KeywordService>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MealHTML {
    pub title: String,
    pub foodies: Vec<Foody>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Foody {
    #[serde(rename = "type")]
    pub r#type: String,
    pub content: Vec<String>,
}

impl MealsAction {
    pub fn new(
        meal_service: Arc<MealService>,
        restaurants_service: Arc<RestaurantService>,
        keyword_service: Arc<KeywordService>,
    ) -> Self {
        Self {
            meal_service,
            restaurants_service,
            keyword_service,
        }
    }
}

#[async_trait]
impl Action for MealsAction {
    async fn execute(&self) -> Result<ExitResult, ExitResult> {
        let restaurants = self
            .restaurants_service
            .find_all()
            .await
            .map(|restaurants| {
                let mut restaurants_url: Vec<Restaurant> = Vec::new();
                for restaurant in restaurants.iter() {
                    if restaurant.url.is_empty() || restaurant.idrestaurant.is_none() {
                        continue;
                    }
                    restaurants_url.push(restaurant.clone());
                }
                restaurants_url
            })
            .map_err(|e| {
                ExitResult {
                    exit_code: ExitCode::from(2),
                    message: format!("can't find restaurants: {}", e),
                }
            });

        let tasks: Vec<_> = match restaurants {
            Ok(restaurants) => restaurants
                .into_iter()
                .map(|restaurant| {
                    tokio::spawn(async move {
                        match scrape_meals(restaurant.clone()).await {
                            Ok(meals) => {
                                info!("[{}] menu found", restaurant.name);
                                meals
                            },
                            Err(err) => {
                                match err {
                                    MealError::DomIssue(element) => {
                                        error!("[{}] couldn't find element in DOM : {}",restaurant.name,  element);
                                    }
                                    MealError::NoMenuFound => {
                                        error!("[{}] no menu found", restaurant.name);
                                    }
                                    MealError::NoDateFound => {
                                        error!("[{}] no date found", restaurant.name);
                                    }
                                    MealError::Reqwest(message) => {
                                        error!("[{}] {}",restaurant.name, message);
                                    }
                                };
                                Vec::new()
                            },
                        }
                    })
                })
                .collect(),
            Err(exit_result) => return Err(exit_result),
        };

        match self.meal_service.clean().await {
            Ok(_) => (),
            Err(err) => {
                return Err(ExitResult {
                    exit_code: ExitCode::from(2),
                    message: format!("clear failed: {}", err),
                })
            }
        }

        for task in tasks {
            match task.await {
                Ok(meals) => {
                    for meal in meals {
                        match self.meal_service.create(&meal).await {
                            Ok(_) => {
                                for foodies in meal.foodies.iter() {
                                    for content in foodies.content.iter() {
                                        self.keyword_service
                                            .create(
                                                content.clone(),
                                                meal.idrestaurant,
                                                Category::Food,
                                            )
                                            .await
                                            .map_err(|e| {
                                                ExitResult {
                                                    exit_code: ExitCode::from(2),
                                                    message: format!("can't create keyword: {}", e),
                                                }
                                            })?;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("error: {}", e);
                            }
                        }
                    }
                }
                Err(_) => {
                    error!("error");
                }
            }
        }

        // Collect the results and handle any errors

        Ok(ExitResult {
            exit_code: ExitCode::from(0),
            message: "meals done".to_string(),
        })
    }

    fn help(&self) -> &str {
        "scrape meals on all restaurants available in the given database"
    }
}

pub enum MealError {
    NoMenuFound,
    NoDateFound,
    DomIssue(String),
    Reqwest(String)
}

async fn scrape_meals(restaurant: Restaurant) -> Result<Vec<Meal>, MealError> {
    let url = restaurant.url;
    let id = restaurant.idrestaurant.unwrap();
    let resp = reqwest::get(url)
        .await
        .map_err(|e| MealError::Reqwest(format!("Reqwest error : {}", e)))?
        .text()
        .await
        .map_err(|e| MealError::Reqwest(format!("Reqwest to text error : {}", e)))?;
    let document = Html::parse_document(&resp);
    let menu_selector = Selector::parse(".menu").map_err(|_| MealError::NoMenuFound)?;
    let menu_element = document.select(&menu_selector);
    let date_selector = Selector::parse(".menu_date_title").map_err(|_| MealError::NoMenuFound)?;
    let date_element = menu_element
        .clone()
        .next()
        .ok_or(MealError::NoDateFound)?;

    let date_element = date_element.select(&date_selector);

    let date = date_element
        .into_iter()
        .next()
        .ok_or(MealError::NoDateFound)?
        .text()
        .collect::<String>();

    let meal_selector = Selector::parse(".meal").map_err(|_| MealError::DomIssue(".meal".to_string()))?;
    let meal_element = menu_element
        .clone()
        .next()
        .ok_or(MealError::NoMenuFound)?
        .select(&meal_selector);

    let mut meals: Vec<Meal> = Vec::new();

    for meal in meal_element {
        // select .meal_title inside of meal
        let meal_title_selector = Selector::parse(".meal_title").map_err(|_| MealError::DomIssue(".meal_title".to_string()))?;
        let meal_title_element = meal.select(&meal_title_selector);
        let meal_title = meal_title_element
            .into_iter()
            .next()
            .ok_or(MealError::NoMenuFound)?
            .text()
            .collect::<String>();

        let meal_foodies_selector = Selector::parse("ul.meal_foodies > li").map_err(|_| MealError::DomIssue("ul.meal_foodies > li".to_string()))?;

        let meal_foodies_element = meal.select(&meal_foodies_selector);

        let mut meal_foodies: Vec<Foody> = Vec::new();
        for meal_foodie in meal_foodies_element {
            // get first element of meal foodie inner html after spliting by <ul>
            let meal_foodie_title = meal_foodie
                .inner_html()
                .split("<ul>")
                .next()
                .unwrap()
                .to_string();

            let foodie_content_selector = Selector::parse("ul li").map_err(|_| MealError::DomIssue("ul li".to_string()))?;
            let foodie_content_element = meal_foodie.select(&foodie_content_selector);

            let mut foodie_content: Vec<String> = Vec::new();

            for foodie in foodie_content_element {
                let foodie_text = foodie.text().collect::<String>();
                foodie_content.push(foodie_text);
            }

            meal_foodies.push(Foody {
                r#type: meal_foodie_title,
                content: foodie_content,
            });
        }
        let meal_html = MealHTML {
            title: meal_title,
            foodies: meal_foodies,
        };

        meals.push(Meal {
            day: parse_date(date.clone()),
            typemeal: meal_html.title,
            foodies: sqlx::types::Json(meal_html.foodies),
            idrestaurant: i64::from(id),
        })
    }

    Ok(meals)
}

fn parse_date(date: String) -> chrono::DateTime<chrono::Utc> {
    let months = [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ];
    let mut month: u32 = 0;
    let mut day: u32 = 0;
    let mut year: u32 = 0;
    for (index, word) in date.split_whitespace().enumerate() {
        if months.contains(&word) {
            day = date
                .split_whitespace()
                .nth(index - 1)
                .unwrap()
                .parse()
                .unwrap();
            month = months.iter().position(|&r| r == word).unwrap() as u32 + 1;
            year = date
                .split_whitespace()
                .nth(index + 1)
                .unwrap()
                .parse()
                .unwrap();
        }
    }
    chrono::Utc
        .with_ymd_and_hms(year as i32, month, day, 0, 0, 0)
        .unwrap()
}
