use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use sqlx::{postgres::PgRow, Error, FromRow, Pool, Postgres};

use crate::{
    database_definition::table_definition::DatabaseTableDefinition,
    queries::{Insertable, Query, Queryable},
    AsSql,
};

#[derive(Clone)]
pub struct PostgresDataProvider<T: Queryable + Insertable> {
    pub table_definition: DatabaseTableDefinition,
    pub db_pool: Pool<Postgres>,
    pub _t: PhantomData<T>,
}

pub trait GetTableDefinition {
    fn get_table_definition() -> DatabaseTableDefinition;
}

/// Wraps a `Query<T>` alongside a DB Pool (using sqlx), to enable ergonomic querying.
pub struct ExecutableQuery<T: Queryable + Insertable> {
    query: Query<T>,
    db_pool: Pool<Postgres>,
}

impl<T: Queryable + Insertable> Deref for ExecutableQuery<T> {
    type Target = Query<T>;
    fn deref(&self) -> &Self::Target {
        &self.query
    }
}

impl<T: Queryable + Insertable> DerefMut for ExecutableQuery<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query
    }
}

// This is a fun mess of requirements inherited for T
impl<T: Queryable + Insertable + for<'r> FromRow<'r, PgRow> + Send + Unpin> ExecutableQuery<T> {
    pub async fn execute(self) -> Result<Vec<T>, String> {
        let sql = self.query.as_sql();
        log::debug!("Executing Query: {}", &sql);
        // let result = sqlx::query(&sql).fetch_all(&self.db_pool).await;
        let result: Result<Vec<T>, Error> = sqlx::query_as(&sql).fetch_all(&self.db_pool).await;
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

impl<T: Queryable + Insertable> PostgresDataProvider<T> {
    pub fn all(&self) -> ExecutableQuery<T> {
        let query = Query::<T> {
            table: self.table_definition.clone(),
            filter: None,
            limit: None,
            _t: Default::default(),
        };
        ExecutableQuery {
            query,
            db_pool: self.db_pool.clone(),
        }
    }

    pub async fn create<I: Insertable>(
        &self,
        item: &I,
    ) {
        let insert = item.get_insert_statement();
        sqlx::query(&insert.as_sql()).execute(&self.db_pool).await.unwrap();
    }
}
