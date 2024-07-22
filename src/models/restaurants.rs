use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

#[derive(Clone)]
pub struct RestaurantService {
    pool: Arc<PgPool>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Restaurant {
    idrestaurant: String,
    url: String,
    name: String,
    gpscoord: Option<String>,
    hours: Option<String>,
}

impl RestaurantService {
    pub async fn find_all(&self) -> Result<Vec<Restaurant>, sqlx::Error> {
        let restaurants =
            sqlx::query_as::<_, Restaurant>("SELECT idrestaurant, url, name FROM restaurant")
                .fetch_all(self.pool.as_ref())
                .await?;
        Ok(restaurants)
    }

    pub async fn create(&self, restaurant: Restaurant) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO restaurant(url, name, gpscord, hours) VALUES ($1, $2, $3, $4)")
            .bind(restaurant.url)
            .bind(restaurant.name)
            .bind(restaurant.gpscoord)
            .bind(restaurant.hours)
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }
}
