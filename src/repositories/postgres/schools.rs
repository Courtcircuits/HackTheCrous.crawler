use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

use crate::repositories::school::{SchoolError, SchoolService};

#[derive(Clone)]
pub struct PostgresSchoolService {
    pub pool: Arc<PgPool>,
}

#[derive(Debug, FromRow, Serialize, Clone, Deserialize)]
pub struct School {
    pub idschool: i64,
    pub long_name: String,
    pub name: String,
    pub coords: String,
}

impl PostgresSchoolService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl SchoolService for PostgresSchoolService {
    async fn create(&self, school: School) -> Result<(), SchoolError> {
        sqlx::query(
            format!(
                "INSERT INTO school(long_name, name, coords) VALUES ($1, $2, '{}')",
                school.coords.to_string().as_str()
            )
            .as_str(),
        )
        .bind(school.long_name)
        .bind(school.name)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| SchoolError::DatabaseError(e))?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), SchoolError> {
        sqlx::query("DELETE FROM school")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| SchoolError::DatabaseError(e))?;
        Ok(())
    }
}
