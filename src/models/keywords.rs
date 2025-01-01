use std::sync::Arc;

use sqlx::PgPool;

pub struct Keyword {
    pub idsuggestion: i64,
    pub keyword: String,
    pub idrestaurant: i64,
    pub idcat: i64,
}

#[derive(Clone)]
pub struct KeywordService {
    pub pool: Arc<PgPool>,
}

pub enum Category {
    Meal,
    Restaurant,
    Food,
}

impl Category {
    pub fn to_int(&self) -> i64 {
        match self {
            Category::Meal => 1,
            Category::Restaurant => 2,
            Category::Food => 3,
        }
    }
}

impl KeywordService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
    pub async fn create(
        &self,
        keyword: String,
        idrestaurant: i64,
        category: Category,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO suggestions_restaurant(keyword, idrestaurant, idcat) VALUES ($1, $2, $3)"#,
        )
        .bind(keyword)
        .bind(idrestaurant)
        .bind(category.to_int())
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}
