use crate::AsSql;

use crate::data_definition::table::DatabaseTableDefinition;
use crate::queries::Filter;

pub struct DeleteStatement {
    table_def: DatabaseTableDefinition,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    filter: Filter,
}

impl AsSql for DeleteStatement {
    fn as_sql(&self) -> String {
        // TODO: Clean  this up a bit?
        format!("DELETE FROM {} WHERE {};", &self.table_def.table_name, &self.filter.as_sql())
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
