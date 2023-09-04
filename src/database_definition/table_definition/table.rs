use std::{collections::HashMap, ops::Deref, sync::Arc};

use super::{Identifier, TableColumn, TableConstraint};

// The details of the Database table. Used to generate the queries for setting up and iteracting with the database.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DatabaseTableDefinition {
    data: Arc<DatabaseTableDefinitionData>,
}

impl Deref for DatabaseTableDefinition {
    type Target = DatabaseTableDefinitionData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DatabaseTableDefinition {
    pub fn new(table_name: &str) -> Result<DatabaseTableDefinitionData, String> {
        DatabaseTableDefinitionData::new(table_name)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DatabaseTableDefinitionData {
    pub table_name: Identifier,
    // TODO: Make it so that there can only be one ID column.
    // TODO: Composite keys, Constraints, etc.
    // pub columns: Vec<TableColumn>,
    pub columns: HashMap<Identifier, TableColumn>,
    pub constraints: Vec<TableConstraint>,
}

impl Into<DatabaseTableDefinition> for DatabaseTableDefinitionData {
    fn into(self) -> DatabaseTableDefinition {
        DatabaseTableDefinition {
            data: Arc::new(self),
        }
    }
}

impl DatabaseTableDefinitionData {
    pub fn new(table_name: &str) -> Result<Self, String> {
        Ok(Self {
            table_name: Identifier::new(table_name)?, // TODO: Clean this up
            columns: HashMap::new(),
            constraints: vec![],
        })
    }

    pub fn column<T: Into<TableColumn>>(
        self,
        column: T,
    ) -> Self {
        self.add_column(column)
    }

    pub fn add_column<T: Into<TableColumn>>(
        mut self,
        column: T,
    ) -> Self {
        let column = column.into();
        self.columns.insert(column.column_name.clone(), column);
        self
    }
}
