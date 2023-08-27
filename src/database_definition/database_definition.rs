use std::{ops::Deref, sync::Arc};

use super::table_definition::{DatabaseTableDefinition, Identifier};

pub struct DatabaseDefinition {
    data: Arc<DatabaseDefinitionData>,
}

impl Deref for DatabaseDefinition {
    type Target = DatabaseDefinitionData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DatabaseDefinition {
    pub fn new(
        name: String,
        tables: Vec<DatabaseTableDefinition>,
    ) -> Self {
        Self {
            data: Arc::new(DatabaseDefinitionData {
                name: Identifier::new(name).unwrap(), // TODO: remove `.unwrap` calls
                tables,
            }),
        }
    }
}

#[derive(Clone)]
pub struct DatabaseDefinitionData {
    pub name: Identifier,
    pub tables: Vec<DatabaseTableDefinition>,
}

// pub struct DatabaseBuilder {
//     name: String,
//     tables: Vec<DatabaseTableDefinition
// }

impl DatabaseDefinitionData {
    pub fn new(name: String) -> Self {
        Self {
            name: Identifier::new(name).expect("Invalid Database name - panicking"), // TODO: Throw this error upstream
            tables: Vec::new(),
        }
    }

    pub fn add_table(
        mut self,
        table_name: String,
    ) -> Self {
        let new_table = DatabaseTableDefinition {
            table_name: Identifier::new(table_name).expect("Invalid Table name - panicking"), // TODO: Throw this error upstream
            columns: Vec::new(),
        };
        self.tables.push(new_table);
        self
    }
}
