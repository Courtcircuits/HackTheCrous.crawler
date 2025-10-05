use crate::repositories::postgres::restaurants::Restaurant;
use async_trait::async_trait;

pub trait RestaurantService {
    async fn find_all(&self) -> Result<Vec<Restaurant>, RestaurantError>;
    async fn create(&self, restaurant: Restaurant) -> Result<Restaurant, RestaurantError>;
    async fn clear(&self) -> Result<(), RestaurantError>;
}

#[derive(Debug)]
pub enum RestaurantError {
    DatabaseError(sqlx::Error),
}
