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
    pub fn new(name: &str) -> Result<DatabaseDefinitionData, String> {
        DatabaseDefinitionData::new(name)
    }
}

#[derive(Clone)]
pub struct DatabaseDefinitionData {
    pub name: Identifier,
    pub tables: Vec<DatabaseTableDefinition>,
}

impl Into<DatabaseDefinition> for DatabaseDefinitionData {
    fn into(self) -> DatabaseDefinition {
        DatabaseDefinition {
            data: Arc::new(self),
        }
    }
}

impl DatabaseDefinitionData {
    pub fn new(name: &str) -> Result<Self, String> {
        Ok(Self {
            name: Identifier::new(name)?,
            tables: Vec::new(),
        })
    }

    pub fn add_table(
        mut self,
        table: DatabaseTableDefinition,
    ) -> Self {
        self.tables.push(table);
        self
    }

    pub fn table(
        self,
        table: DatabaseTableDefinition,
    ) -> Self {
        self.add_table(table)
    }
}
