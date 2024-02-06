use std::marker::PhantomData;

use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::{
    data_manager::{GetTableDefinition, PostgresDataProvider},
    queries::Insertable,
};

pub trait ConnectPostgres
where
    Self: Insertable + GetTableDefinition + Sized,
{
    fn connect_postgres(db_pool: Pool<Postgres>) -> PostgresDataProvider<Self>;
}

#[async_trait]
impl<T> ConnectPostgres for T
where
    Self: Insertable + GetTableDefinition,
{
    fn connect_postgres(db_pool: Pool<Postgres>) -> PostgresDataProvider<Self> {
        PostgresDataProvider {
            table_definition: Self::get_table_definition(),
            db_pool,
            _t: PhantomData,
        }
    }
}
