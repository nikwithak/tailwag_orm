use crate::BuildSql;

use crate::data_definition::table::DatabaseTableDefinition;
use crate::queries::Filter;

pub struct DeleteStatement<T> {
    table_def: DatabaseTableDefinition<T>,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    filter: Filter,
}

impl<T> BuildSql for DeleteStatement<T> {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        builder.push(format!("DELETE FROM {} WHERE ", &self.table_def.table_name));
        self.filter.build_sql(builder);
    }
}

impl<T> DeleteStatement<T> {
    pub fn new(
        table_def: DatabaseTableDefinition<T>,
        filter: Filter,
    ) -> Self {
        Self {
            table_def,
            filter,
        }
    }
}
