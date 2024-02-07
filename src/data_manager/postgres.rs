use crate::{
    data_definition::{database_definition::DatabaseDefinition, table::DatabaseTableDefinition},
    migration::Migration,
    queries::{filterable_types::Filterable, Deleteable, Filter, Insertable, Query, Updateable},
    AsSql, BuildSql,
};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, Error, FromRow, Pool, Postgres};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    vec::IntoIter,
};

use super::{rest_api::Id, traits::WithFilter};

#[derive(Clone)]
pub struct PostgresDataProvider<T: Insertable> {
    pub table_definition: DatabaseTableDefinition<T>,
    pub db_pool: Pool<Postgres>,
    pub _t: PhantomData<T>,
}

pub trait GetTableDefinition {
    fn get_table_definition() -> DatabaseTableDefinition<Self>
    where
        Self: std::marker::Sized;
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
            filter.build_sql(&mut query_builder);
        }
        // stream.push()

        let result = query_builder.build_query_as::<T>().fetch_all(&self.db_pool).await?;
        Ok(result)
    }
}

impl<T: Filterable> ExecutableQuery<T> {
    pub fn with_filter<F>(
        mut self,
        derive_filter: F,
    ) -> Self
    where
        F: Fn(T::FilterType) -> Filter,
    {
        let filter = derive_filter(T::FilterType::default());
        self.query = self.query.filter(filter);
        self
    }
}
// Migration Handling
impl<T: Insertable> PostgresDataProvider<T>
where
    T: Clone + std::fmt::Debug,
{
    fn build_migration(&self) -> Option<Migration<T>> {
        Migration::<T>::compare(
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
            match sqlx::query::<Postgres>(dbg!(&migration.as_sql())).execute(&self.db_pool).await {
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

// #[async_trait]
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
        + Filterable
        + Default,
{
    type CreateRequest = T; // TODO: Implement this based ont he implementaiton of Insertable?
    type Error = crate::Error;

    async fn get(
        &self,
        predicate: impl Fn(<T as Filterable>::FilterType) -> crate::queries::Filter,
    ) -> Result<Option<T>, Self::Error> {
        let query = Query::<T> {
            table: self.table_definition.clone(),
            filter: Some(predicate(<T as Filterable>::FilterType::default())),
            limit: Some(2),
            _t: Default::default(),
        };
        let query = ExecutableQuery {
            query,
            db_pool: self.db_pool.clone(),
        };
        let mut results = query.execute().await?;
        if results.len() > 1 {
            return Err(Self::Error::DataIntegrity("Multiple items found for ID {}".to_string()));
        }
        Ok(results.pop())
    }

    async fn all(&self) -> Result<impl Iterator<Item = T>, Self::Error> {
        let query = Query::<T> {
            table: self.table_definition.clone(),
            filter: None,
            limit: None,
            _t: Default::default(),
        };
        let query = ExecutableQuery {
            query,
            db_pool: self.db_pool.clone(),
        };
        Ok(query.execute().await?.into_iter())
    }

    async fn create(
        &self,
        item: Self::CreateRequest,
    ) -> Result<T, Self::Error> {
        let insert = item.get_insert_statement();
        // TODO/DEBUG: Is the query returning the result? Can I simply run it as a query_as instead?
        sqlx::query(&insert.as_sql()).execute(&self.db_pool).await?;
        Ok(item)
    }

    async fn delete(
        &self,
        item: T,
    ) -> Result<(), Self::Error> {
        let mut builder = sqlx::QueryBuilder::new("");
        item.get_delete_statement().build_sql(&mut builder);
        sqlx::query(&builder.into_sql()).execute(&self.db_pool).await?;
        Ok(())
    }

    async fn update(
        &self,
        item: &T,
    ) -> Result<(), Self::Error> {
        let update = item.get_update_statement();
        sqlx::query(&update.as_sql()).execute(&self.db_pool).await?;
        Ok(())
    }
}

impl<T> WithFilter<T> for PostgresDataProvider<T>
where
    T: Filterable + Insertable + Clone,
{
    type R = ExecutableQuery<T>;
    fn with_filter(
        &self,
        predicate: impl Fn(<T as Filterable>::FilterType) -> crate::queries::Filter,
    ) -> Self::R {
        let query = Query::<T> {
            table: self.table_definition.clone(),
            filter: Some(predicate(T::FilterType::default())),
            limit: None,
            _t: Default::default(),
        };
        ExecutableQuery {
            query,
            db_pool: self.db_pool.clone(),
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
