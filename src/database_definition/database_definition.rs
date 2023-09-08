use std::{ops::Deref, sync::Arc};

use super::table_definition::{DatabaseTableDefinition, Identifier};

#[derive(Debug)]
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
    /// Creates an empty DatabaseDefinitionData object.
    ///
    /// DatabaseDefinitionData uses the builder pattern to build the object,
    /// and is locked into an [Arc] wrapped in `DatabaseDefinition` with .into()
    ///
    /// Example:
    /// ```
    /// let database_definition: DatabaseDefinition
    ///     = DatabaseDefinition::new("new_database")
    ///         .expect("Name is not allowed")
    ///         .into();
    /// ```
    pub fn new(name: &str) -> Result<DatabaseDefinitionData, String> {
        DatabaseDefinitionData::new(name)
    }

    /// Creates an empty DatabaseDefinitionData object.  Does not wrap the result in a Result,
    /// instead this method panics if `name` is an invalid identifier.
    /// Example:
    /// ```
    /// let database_definition: DatabaseDefinition
    ///     = DatabaseDefinition::new("new_database").into();
    /// ```
    pub fn new_unchecked(name: &str) -> DatabaseDefinitionData {
        match DatabaseDefinitionData::new(name) {
            Ok(def) => def,
            Err(e) => panic!("DatabaseDefinition::new_unchecked() failed: {}", e),
        }
    }
}

#[derive(Debug, Clone)]
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

    pub fn table<T: Into<DatabaseTableDefinition>>(
        self,
        table: T,
    ) -> Self {
        self.add_table(table.into())
    }
}
