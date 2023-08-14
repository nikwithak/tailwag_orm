use std::{collections::HashMap, sync::Arc};

use syn::Ident;

use crate::{
    database_definition::table_definition::{self, DatabaseTableDefinition, Identifier},
    AsSql,
};

pub struct InsertStatement {
    table_def: Arc<DatabaseTableDefinition>,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    object_repr: HashMap<Arc<Identifier>, String>,
}

impl InsertStatement {
    pub fn new(
        table_def: Arc<DatabaseTableDefinition>,
        object_map: HashMap<Arc<Identifier>, String>,
    ) -> Self {
        Self {
            table_def,
            object_repr: object_map,
        }
    }
}

impl AsSql for InsertStatement {
    fn as_sql(&self) -> String {
        let mut columns: Vec<&str> = Vec::new();
        let mut values = Vec::new();
        for (column, value) in &self.object_repr {
            columns.push(column);
            values.push(value.as_str());
        }
        let insert_statement = format!(
            // TODO: Need to prepare this statement, to avoid SQL Injection
            "INSERT INTO {} ({}) VALUES ({});",
            self.table_def.table_name,
            &columns.join(", "),
            &values.join(", "),
        );

        insert_statement
    }
}
