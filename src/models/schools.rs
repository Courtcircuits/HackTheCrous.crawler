use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

#[derive(Clone)]
pub struct SchoolService {
    pub pool: Arc<PgPool>,
}

#[derive(Debug, FromRow, Serialize, Clone, Deserialize)]
pub struct School {
    pub idschool: i64,
    pub long_name: String,
    pub name: String,
    pub coords: String,
}

impl SchoolService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, school: School) -> Result<(), sqlx::Error> {
        sqlx::query(format!(
            "INSERT INTO school(long_name, name, coords) VALUES ($1, $2, '{}')",
            school.coords.to_string().as_str()
        ).as_str())
        .bind(school.long_name)
        .bind(school.name)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM school")
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }
}
