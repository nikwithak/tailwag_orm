use std::{collections::HashMap, sync::Arc};

use crate::AsSql;

use crate::database_definition::table_definition::{DatabaseTableDefinition, Identifier};
use crate::queries::Filter;

pub struct UpdateStatement {
    table_def: Arc<DatabaseTableDefinition>,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    object_repr: HashMap<Arc<Identifier>, String>,
    filter: Filter,
}

impl AsSql for UpdateStatement {
    fn as_sql(&self) -> String {
        let mut columns: Vec<&str> = Vec::new();
        let mut values = Vec::new();

        for (column, value) in &self.object_repr {
            if column.value().eq("id") {
                continue;
            }
            columns.push(column);
            values.push(value.as_str());
        }
        let insert_statement = format!(
            // TODO: Need to prepare this statement, to avoid SQL Injection
            "UPDATE {} SET ({}) = ({}) WHERE {};",
            self.table_def.table_name,
            &columns.join(", "),
            &values.join(", "),
            self.filter.as_sql(),
        );

        insert_statement
    }
}
