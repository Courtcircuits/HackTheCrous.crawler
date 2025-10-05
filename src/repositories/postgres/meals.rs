use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    cli::actions::meals::Foody,
    repositories::meals::{MealError, MealService},
};

#[derive(Clone)]
pub struct PostgresMealService {
    pub pool: Arc<PgPool>,
}

#[derive(Debug)]
pub struct Meal {
    pub typemeal: String,
    pub foodies: sqlx::types::Json<Vec<Foody>>,
    pub day: chrono::DateTime<chrono::Utc>,
    pub idrestaurant: i64,
}

impl PostgresMealService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl MealService for PostgresMealService {
    async fn create(&self, meal: &Meal) -> Result<(), MealError> {
        sqlx::query(
            r#"INSERT INTO meal(typemeal, foodies, day, idrestaurant) VALUES ($1, $2, $3, $4)"#,
        )
        .bind(&meal.typemeal)
        .bind(&meal.foodies)
        .bind(meal.day)
        .bind(meal.idrestaurant)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| MealError::DatabaseError(e))?;
        Ok(())
    }

    async fn clean(&self) -> Result<(), MealError> {
        sqlx::query("DELETE FROM meal")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| MealError::DatabaseError(e))?;
        Ok(())
    }
}
