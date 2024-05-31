use crate::{
    data_definition::{database_definition::DatabaseDefinition, table::DatabaseTableDefinition},
    migration::Migration,
    queries::{filterable_types::Filterable, Deleteable, Filter, Insertable, Query, Updateable},
    BuildSql,
};
use sqlx::Row;
use sqlx::{Error, Execute, Pool, Postgres, QueryBuilder};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::{rest_api::Id, traits::WithFilter};

#[derive(Clone)]
pub struct PostgresDataProvider<T: Insertable> {
    pub table_definition: DatabaseTableDefinition<T>,
    pub db_pool: Pool<Postgres>,
    pub _t: PhantomData<T>,
}

impl<T> PostgresDataProvider<T>
where
    T: Insertable,
{
    pub fn new(
        table_definition: DatabaseTableDefinition<T>,
        db_pool: Pool<Postgres>,
    ) -> Self {
        Self {
            table_definition,
            db_pool,
            _t: PhantomData,
        }
    }
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
    T: Insertable + for<'r> serde::Deserialize<'r> + Send + Sync + Unpin,
{
    fn from(val: ExecutableQuery<T>) -> Self {
        futures::executor::block_on(val.execute()).unwrap()
    }
}

impl<T: Insertable + for<'d> serde::Deserialize<'d> + Send + Sync + Unpin> ExecutableQuery<T> {
    pub async fn execute(self) -> Result<Vec<T>, Error> {
        // We wrap the whole thing in a `to_json` on the DB side. This makes it supes easy to deserialize.
        // Without his, it got really messy, because it seems that SQLX doesn't support nested deserialization on its own.
        // A little bit more overhead, perhaps, but jeeeeez does it save on dvelopment. And postgres is probably
        // not the limiting factor rn anyway.
        let mut query_builder = QueryBuilder::new("SELECT to_json(r) as json_result FROM (");
        // let mut query_builder = QueryBuilder::new("");
        self.build_sql(&mut query_builder);
        query_builder.push(") r");

        log::debug!("SQL query: {}", query_builder.sql());
        let result = query_builder.build().fetch_all(&self.db_pool).await?;
        let results = result.into_iter().map(|row| {
            let rowresult: serde_json::Value = row.get("json_result");
            log::debug!("Result: {:?}", rowresult.to_string());
            serde_json::from_value::<T>(rowresult).unwrap()
        });
        Ok(results.collect())
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
    T: Clone + std::fmt::Debug + for<'d> serde::Deserialize<'d>,
{
    pub fn build_migration(&self) -> Option<Migration<T>> {
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
            let mut sql = sqlx::QueryBuilder::new("");
            migration.build_sql(&mut sql);

            let sql = sql.into_sql();
            let mut transaction = self.db_pool.begin().await.unwrap();
            match sqlx::query(dbg!(&sql)).execute(&mut *transaction).await {
                Ok(_) => {},
                Err(e) => {
                    log::error!("Failed to run migrations");
                    return Err(format!("{}", e));
                },
            }
            transaction.commit().await.unwrap();
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
        + Send
        + Sync
        + serde::Serialize
        + for<'d> serde::Deserialize<'d>
        + Clone
        + Unpin
        + Id
        // + Serialize
        + Filterable
        + Default,
{
    type CreateRequest = T::CreateRequest; // TODO: Implement this based ont he implementaiton of Insertable?
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
        let item = item.into();
        let insert_statements = item.get_insert_statement();

        let mut transaction = self.db_pool.begin().await?;
        for insert in insert_statements {
            let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new("");
            insert.build_sql(&mut builder);
            builder.build().execute(&mut *transaction).await?;
        }
        transaction.commit().await?;

        Ok(item)
    }

    async fn delete(
        &self,
        item: T,
    ) -> Result<(), Self::Error> {
        let mut builder: QueryBuilder<'_, Postgres> = sqlx::QueryBuilder::new("");
        item.get_delete_statement().build_sql(&mut builder);
        let query = builder.build();
        dbg!(&query.sql());
        if query.execute(&self.db_pool).await?.rows_affected() > 1 {
            panic!("Deleted more than one row in a Delete operation. This should not happen.");
        }

        Ok(())
    }

    async fn update(
        &self,
        item: &T,
    ) -> Result<(), Self::Error> {
        let mut transaction = self.db_pool.begin().await?;
        let update_statements = item.get_update_statement();
        for update in update_statements {
            let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new("");
            update.build_sql(&mut builder);
            builder.build().execute(&mut *transaction).await?;
        }
        transaction.commit().await?;

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
