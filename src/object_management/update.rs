use std::collections::HashMap;

use crate::AsSql;

use crate::data_definition::table::{DatabaseTableDefinition, Identifier};

pub struct UpdateStatement<T> {
    table_def: DatabaseTableDefinition<T>,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    object_repr: HashMap<Identifier, String>,
}

impl<T> UpdateStatement<T> {
    pub fn new(
        table_def: DatabaseTableDefinition<T>,
        object_map: HashMap<Identifier, String>,
    ) -> Self {
        Self {
            table_def,
            object_repr: object_map,
        }
    }
}

impl<T> AsSql for UpdateStatement<T> {
    fn as_sql(&self) -> String {
        let mut updates = Vec::new();
        let mut id = None;

        for (column, value) in &self.object_repr {
            if column.value().eq("id") {
                id = Some(value.as_str());
            } else {
                updates.push(format!("{}={}", &column, value.as_str()))
            }
        }
        if let Some(id) = id {
            let insert_statement = format!(
                // TODO: Need to prepare this statement, to avoid SQL Injection
                "UPDATE {} SET {} WHERE id = {};",
                self.table_def.table_name,
                &updates.join(", "),
                &id
            );

            insert_statement
        } else {
            panic!("Tried to create update without ID")
        }
    }
}
