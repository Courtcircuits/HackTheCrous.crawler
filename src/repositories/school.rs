use async_trait::async_trait;

use crate::repositories::postgres::schools::School;

pub trait SchoolService {
    async fn create(&self, school: School) -> Result<(), SchoolError>;
    async fn clear(&self) -> Result<(), SchoolError>;
}

#[derive(Debug)]
pub enum SchoolError {
    DatabaseError(sqlx::Error),
}
