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

// impl DatabaseDefinition {
// TODO: Make it so that I can read and parse the existing DB Definition from a file (use serde_json)
//     pub fn from_file(file_path: &str) {
//         todo!()
//     }

//     pub fn to_file(file_path: &str) {
//         todo!()
//     }
//
// TODO: Make it so that I can read and parse the existing DB Definition on a remote DB
//     pub fn from_postgres_database(db_connection_string: &str) {
//         todo!()
//     }
// }

impl DatabaseDefinition {
    pub fn new(
        name: String,
        tables: Vec<DatabaseTableDefinition>,
    ) -> Result<Self, String> {
        Ok(Self {
            data: Arc::new(DatabaseDefinitionData {
                name: Identifier::new(name)?,
                tables,
            }),
        })
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
