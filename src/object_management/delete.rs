use crate::{AsSql, BuildSql};

use crate::data_definition::table::DatabaseTableDefinition;
use crate::queries::Filter;

pub struct DeleteStatement {
    table_def: DatabaseTableDefinition,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    filter: Filter,
}

impl BuildSql for DeleteStatement {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        builder.push(format!("DELETE FROM {} WHERE ", &self.table_def.table_name));
        self.filter.build_sql(builder);
    }
}

impl DeleteStatement {
    pub fn new(
        table_def: DatabaseTableDefinition,
        filter: Filter,
    ) -> Self {
        Self {
            table_def,
            filter,
        }
    }
}
