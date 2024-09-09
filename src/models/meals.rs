use std::sync::Arc;

use sqlx::PgPool;

use crate::cli::actions::meals::{Foody, MealHTML};

#[derive(Clone)]
pub struct MealService {
    pub pool: Arc<PgPool>,
}

#[derive(Debug)]
pub struct Meal {
    pub typemeal: String,
    pub foodies: sqlx::types::Json<Vec<Foody>>,
    pub day: chrono::DateTime<chrono::Utc>,
    pub idrestaurant: i64,
}

impl MealService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
    pub async fn create(&self, meal: &Meal) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO meal(typemeal, foodies, day, idrestaurant) VALUES ($1, $2, $3, $4)"#,
        )
        .bind(&meal.typemeal)
        .bind(&meal.foodies)
        .bind(meal.day)
        .bind(meal.idrestaurant)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn clean(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM meal")
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }
}
