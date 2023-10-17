use std::{collections::HashMap, sync::Arc};

use crate::AsSql;

use crate::data_definition::table::{DatabaseTableDefinition, Identifier};
use crate::queries::Filter;

pub struct UpdateStatement {
    table_def: DatabaseTableDefinition,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    object_repr: HashMap<Identifier, String>,
}

impl UpdateStatement {
    pub fn new(
        table_def: DatabaseTableDefinition,
        object_map: HashMap<Identifier, String>,
    ) -> Self {
        Self {
            table_def,
            object_repr: object_map,
        }
    }
}

impl AsSql for UpdateStatement {
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
