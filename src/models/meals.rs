use std::sync::Arc;

use sqlx::PgPool;

#[derive(Clone)]
pub struct MealService {
    pub pool: Arc<PgPool>,
}

pub struct Meal {
    pub typemeal: String,
    pub foodies: String,
    pub day: String,
    pub idrestaurant: String,
}

impl MealService {
    pub async fn create(&self, meal: Meal) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO meal(typemeal, foodies, day, idrestaurant) VALUES ($1, $2, $3, $4)",
        )
        .bind(meal.typemeal)
        .bind(meal.foodies)
        .bind(meal.day)
        .bind(meal.idrestaurant)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}
