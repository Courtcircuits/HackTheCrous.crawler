use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

use crate::repositories::restaurants::{RestaurantError, RestaurantService};

#[derive(Clone)]
pub struct PostgresRestaurantService {
    pub pool: Arc<PgPool>,
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Restaurant {
    pub idrestaurant: Option<i32>,
    pub url: String,
    pub name: String,
    pub gpscoord: Option<String>,
    pub hours: Option<String>,
}

impl PostgresRestaurantService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl RestaurantService for PostgresRestaurantService {
    async fn find_all(&self) -> Result<Vec<Restaurant>, RestaurantError> {
        let restaurants = sqlx::query_as::<_, Restaurant>(
            r#"SELECT idrestaurant, url, name, gpscoord::text as gpscoord, hours FROM restaurant"#,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| RestaurantError::DatabaseError(e))?;
        Ok(restaurants)
    }

    async fn create(&self, restaurant: Restaurant) -> Result<Restaurant, RestaurantError> {
        if restaurant.gpscoord.is_none() {
            let restaurant_result = sqlx::query_as::<_, Restaurant>(
                "INSERT INTO restaurant(url, name, hours) VALUES ($1, $2, $3) RETURNING idrestaurant, url, name, gpscoord::text as gpscoord, hours",
            )
            .bind(restaurant.url)
            .bind(restaurant.name)
            .bind(restaurant.hours)
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| RestaurantError::DatabaseError(e))?;
            return Ok(restaurant_result);
        }
        let restaurant_result = sqlx::query_as::<_, Restaurant>(
            format!(
                "INSERT INTO restaurant(url, name, hours, gpscoord) VALUES ($1, $2, $3, {}) RETURNING idrestaurant, url, name, gpscoord::text as gpscoord, hours",
                restaurant.gpscoord.unwrap()
            )
            .as_str(),
        )
        .bind(restaurant.url)
        .bind(restaurant.name)
        .bind(restaurant.hours)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| RestaurantError::DatabaseError(e))?;
        Ok(restaurant_result)
    }

    async fn clear(&self) -> Result<(), RestaurantError> {
        sqlx::query("DELETE FROM restaurant")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RestaurantError::DatabaseError(e))?;
        Ok(())
    }
}
