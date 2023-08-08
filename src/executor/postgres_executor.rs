use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use uuid::Uuid;

use crate::{
    database_definition::{
        database_definition::DatabaseDefinition, table_definition::DatabaseTableDefinition,
    },
    queries::Queryable,
};

struct PostgresDataManager<T> {
    table_definition: DatabaseTableDefinition,
    db_pool: Pool<Postgres>,
    _t: Option<T>,
}

impl<T> PostgresDataManager<T> {
    pub async fn init(
        table_definition: DatabaseTableDefinition,
        conn_string: &str,
    ) -> Self {
        Self {
            table_definition,
            db_pool: PgPoolOptions::new().max_connections(1).connect(conn_string).await.unwrap(),
            _t: None,
        }
    }

    pub async fn get(
        &self,
        id: Uuid,
    ) -> Option<T> {
        todo!()
    }
}
