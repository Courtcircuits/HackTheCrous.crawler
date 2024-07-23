use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

#[derive(Clone)]
pub struct RestaurantService {
    pub pool: Arc<PgPool>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Restaurant {
    pub idrestaurant: Option<String>,
    pub url: String,
    pub name: String,
    pub gpscoord: Option<String>,
    pub hours: Option<String>,
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
        sqlx::query("INSERT INTO restaurant(url, name) VALUES ($1, $2)")
            .bind(restaurant.url)
            .bind(restaurant.name)
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM restaurant")
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }
}
