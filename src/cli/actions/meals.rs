use std::{process::ExitCode, sync::Arc};

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{
    cli::{Action, ExitResult},
    models::{
        keywords::{Category, Keyword, KeywordService},
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
                return restaurants_url;
            })
            .map_err(|e| {
                return ExitResult {
                    exit_code: ExitCode::from(2),
                    message: format!("can't find restaurants: {}", e),
                };
            });

        let tasks: Vec<_> = match restaurants {
            Ok(restaurants) => restaurants
                .into_iter()
                .map(|restaurant| {
                    tokio::spawn(async move {
                        match scrape_meals(restaurant).await {
                            Ok(meals) => meals,
                            Err(_) => Vec::new(),
                        }
                    })
                })
                .collect(),
            Err(exit_result) => return Err(exit_result),
        };

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
                                                meal.idrestaurant.clone(),
                                                Category::Food,
                                            )
                                            .await
                                            .map_err(|e| {
                                                return ExitResult {
                                                    exit_code: ExitCode::from(2),
                                                    message: format!("can't create keyword: {}", e),
                                                };
                                            })?;
                                    }
                                }
                            }
                            Err(e) => {
                                println!("error: {}", e);
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("error");
                }
            }
        }

        // Collect the results and handle any errors

        Ok(ExitResult {
            exit_code: ExitCode::from(1),
            message: "meals done".to_string(),
        })
    }
}

async fn scrape_meals(restaurant: Restaurant) -> Result<Vec<Meal>, Box<dyn std::error::Error>> {
    let url = restaurant.url;
    let id = restaurant.idrestaurant.unwrap();
    let resp = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&resp);
    let menu_selector = Selector::parse(".menu")?;
    let menu_element = document.select(&menu_selector);
    let date_selector = Selector::parse(".menu_date_title")?;
    let date_element = menu_element
        .clone()
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

    let meal_selector = Selector::parse(".meal")?;
    let meal_element = menu_element
        .clone()
        .into_iter()
        .next()
        .unwrap()
        .select(&meal_selector);

    let mut meals: Vec<Meal> = Vec::new();

    for meal in meal_element {
        // select .meal_title inside of meal
        let meal_title_selector = Selector::parse(".meal_title")?;
        let meal_title_element = meal.select(&meal_title_selector);
        let meal_title = meal_title_element
            .into_iter()
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let meal_foodies_selector = Selector::parse("ul.meal_foodies > li")?;

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

            let foodie_content_selector = Selector::parse("ul li")?;
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

        println!("Date : {}", parse_date(date.clone()));
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
    let months = vec![
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
    return chrono::Utc.ymd(year as i32, month, day).and_hms(0, 0, 0);
}
