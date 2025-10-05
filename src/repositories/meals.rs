use crate::repositories::postgres::meals::Meal;

pub trait MealService {
    async fn create(&self, meal: &Meal) -> Result<(), MealError>;
    async fn clean(&self) -> Result<(), MealError>;
}

#[derive(Debug)]
pub enum MealError {
    DatabaseError(sqlx::Error),
}
