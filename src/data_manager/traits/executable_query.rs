use std::ops::{Deref, DerefMut};

use sqlx::{postgres::PgRow, FromRow, Pool, Postgres};
use uuid::Uuid;

use crate::{
    queries::{Filter, Insertable, Query},
    AsSql,
};

pub trait QueryItems<T> {
    fn query(&self) -> ExecutableQuery<T>;
}

pub trait GetItemById<T> {
    fn get_by_id(
        &self,
        id: &Uuid,
    ) -> Result<T, String>;
}

pub trait CreateItem<T> {
    fn create(
        &self,
        item: T,
    ) -> Result<T, String>;
}

/// Wraps a `Query<T>` alongside a DB Pool (using sqlx), to enable ergonomic querying
pub struct ExecutableQuery<T> {
    query: Query<T>,
    db_pool: Pool<Postgres>,
}

impl<T: Insertable> Deref for ExecutableQuery<T> {
    type Target = Query<T>;
    fn deref(&self) -> &Self::Target {
        &self.query
    }
}

impl<T: Insertable> DerefMut for ExecutableQuery<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query
    }
}

// This is a fun mess of requirements inherited for T
impl<T: Insertable + for<'r> FromRow<'r, PgRow> + Send + Unpin> ExecutableQuery<T> {
    #[allow(unused)]
    pub async fn execute(self) -> Result<Vec<T>, String> {
        let sql = self.query.as_sql();
        log::debug!("Executing Query: {}", &sql);
        // let result = sqlx::query(&sql).fetch_all(&self.db_pool).await;
        let result: Result<Vec<T>, sqlx::Error> =
            sqlx::query_as(&sql).fetch_all(&self.db_pool).await;
        match result {
            // TODO:
            Ok(results) => Ok(results.into_iter().map(|row| T::from(row)).collect()),
            Err(e) => {
                log::error!("Error while querying for DB: {}", e);
                Err("error querying DB".to_string())
            },
        }
    }
}