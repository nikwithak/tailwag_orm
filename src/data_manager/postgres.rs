use crate::{
    data_definition::{database_definition::DatabaseDefinition, table::DatabaseTableDefinition},
    migration::Migration,
    queries::{Deleteable, Filter, Insertable, Query, Updateable},
    AsSql,
};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, Error, FromRow, Pool, Postgres, QueryBuilder};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::{rest_api::Id, traits::DataResult};

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

impl<T> From<ExecutableQuery<T>> for Vec<T>
where
    T: Insertable + for<'r> FromRow<'r, PgRow> + Send + Unpin,
{
    fn from(val: ExecutableQuery<T>) -> Self {
        futures::executor::block_on(val.execute()).unwrap()
    }
}

// This is a fun mess of requirements inherited for T
// TODO: Simplify this as much as possible
impl<T: Insertable + for<'r> FromRow<'r, PgRow> + Send + Unpin> ExecutableQuery<T> {
    pub async fn execute(self) -> Result<Vec<T>, Error> {
        let mut query_builder = sqlx::QueryBuilder::new(r"SELECT * FROM ");
        query_builder.push(self.query.table.table_name.to_string());
        if let Some(filter) = &self.filter {
            query_builder.push(" WHERE ");
            filter.build_query(&mut query_builder);
        }
        // stream.push()

        let result = query_builder.build_query_as::<T>().fetch_all(&self.db_pool).await?;
        Ok(result)
    }

    pub async fn _execute(self) -> Result<Vec<T>, String> {
        let sql = self.query.as_sql();
        log::debug!("Executing Query: {}", &sql);
        // let result = sqlx::query(&sql).fetch_all(&self.db_pool).await;
        let result: Result<Vec<T>, Error> = sqlx::query_as(&sql).fetch_all(&self.db_pool).await;
        match result {
            // TODO:
            Ok(results) => Ok(results.into_iter().collect()),
            Err(e) => {
                log::error!("Error while querying for DB: {}", e);
                Err("error querying DB".to_string())
            },
        }
    }
}

impl<T> ExecutableQuery<T> {
    pub fn with_filter<K, F>(
        mut self,
        derive_filter: F,
    ) -> Self
    where
        F: Fn(K) -> Filter,
        K: Default,
    {
        let filter = derive_filter(K::default());
        self.query = self.query.filter(filter);
        self
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
impl<T> super::traits::DataProvider<T> for PostgresDataProvider<T>
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
        + Id
        + Default,
{
    type QueryType = ExecutableQuery<T>;
    type CreateRequest = T; // TODO: Implement this based ont he implementaiton of Insertable?

    async fn get(
        &self,
        _id: uuid::Uuid,
    ) -> DataResult<Option<T>> {
        let all = self.all().await.unwrap().execute().await.unwrap();
        Ok(all.into_iter().find(|i| i.id() == &_id))
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
        item: Self::CreateRequest,
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
