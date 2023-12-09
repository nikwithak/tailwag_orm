use crate::{
    data_definition::{database_definition::DatabaseDefinition, table::DatabaseTableDefinition},
    migration::Migration,
    queries::{Deleteable, Insertable, Query, Updateable},
    AsSql,
};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, Error, FromRow, Pool, Postgres};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::traits::DataResult;

#[derive(Clone)]
pub struct PostgresDataProvider<T: Insertable> {
    pub table_definition: DatabaseTableDefinition,
    pub db_pool: Pool<Postgres>,
    pub _t: PhantomData<T>,
}

pub trait GetTableDefinition {
    fn get_table_definition() -> DatabaseTableDefinition;
}

/// Wraps a `Query<T>` alongside a DB Pool (using sqlx), to enable ergonomic querying.
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

impl<T> Into<Vec<T>> for ExecutableQuery<T>
where
    T: Insertable + for<'r> FromRow<'r, PgRow> + Send + Unpin,
{
    fn into(self) -> Vec<T> {
        futures::executor::block_on(self.execute()).unwrap()
    }
}

// This is a fun mess of requirements inherited for T
// TODO: Simplify this as much as possible
impl<T: Insertable + for<'r> FromRow<'r, PgRow> + Send + Unpin> ExecutableQuery<T> {
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

// Migration Handling
impl<T: Insertable> PostgresDataProvider<T> {
    fn build_migration(&self) -> Option<Migration> {
        Migration::compare(
            None, // TODO: Need to get the old migration
            &DatabaseDefinition::new_unchecked("postgres")
                .table(self.table_definition.clone())
                .into(),
        )
    }
    pub async fn run_migrations(&self) -> Result<(), String> {
        log::info!("[DATABASE] Running Migrations");
        let migration = self.build_migration();
        if let Some(migration) = migration {
            match sqlx::query::<Postgres>(&migration.as_sql()).execute(&self.db_pool).await {
                Ok(_) => {},
                Err(e) => {
                    log::error!("Failed to run migrations");
                    return Err(format!("{}", e));
                },
            }
        } else {
            log::info!("[DATABASE] No Migrations");
        }
        Ok(())
    }
}

#[async_trait]
impl<'a, T> super::traits::DataProvider<T> for PostgresDataProvider<T>
// impl<'a, T> PostgresDataProvider<T>
where
    T: Insertable
        + Deleteable
        + Updateable
        + for<'r> FromRow<'r, PgRow>
        + Send
        + Sync
        + Clone
        + Unpin
        + Default,
{
    type QueryType = ExecutableQuery<T>;
    type CreateRequest = T; // TODO: Implement this based ont he implementaiton of Insertable?

    async fn get(
        &self,
        _id: uuid::Uuid,
    ) -> DataResult<Option<T>> {
        todo!()
    }

    async fn all(&self) -> DataResult<ExecutableQuery<T>> {
        let query = Query::<T> {
            table: self.table_definition.clone(),
            filter: None,
            limit: None,
            _t: Default::default(),
        };
        Ok(ExecutableQuery {
            query,
            db_pool: self.db_pool.clone(),
        })
    }

    async fn create(
        &self,
        // item: Self::CreateRequest,
        item: T,
    ) -> Result<T, std::string::String> {
        let insert = item.get_insert_statement();
        println!("Creating...");
        let ret = match sqlx::query(&insert.as_sql()).execute(&self.db_pool).await {
            Ok(_) => Ok(item),
            Err(e) => {
                log::error!("Failed to create item");
                Err(e.to_string())
            },
        };
        println!("Created");
        ret
    }

    async fn delete(
        &self,
        item: T,
    ) -> Result<(), String> {
        let delete = item.get_delete_statement();
        match sqlx::query(&delete.as_sql()).execute(&self.db_pool).await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Failed to delete item");
                Err(e.to_string())
            },
        }
    }

    async fn update(
        &self,
        item: &T,
    ) -> Result<(), String> {
        let update = item.get_update_statement();
        match sqlx::query(&update.as_sql()).execute(&self.db_pool).await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Failed to create item");
                Err(e.to_string())
            },
        }
    }
}

pub trait BuildDataProvider
where
    Self: Sized + Insertable,
{
    fn build_data_provider(&self) -> PostgresDataProvider<Self>;
}

impl<T> BuildDataProvider for T
where
    T: Sized + Insertable,
{
    fn build_data_provider(&self) -> PostgresDataProvider<Self> {
        todo!()
    }
}