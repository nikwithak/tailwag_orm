use super::{DatabaseColumnType, Identifier, TableColumn};

// The details of the Database table. Used to generate the queries for setting up and iteracting with the database.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DatabaseTableDefinition {
    pub table_name: Identifier,
    // TODO: Make it so that there can only be one ID column.
    // TODO: Composite keys, Constraints, etc.
    pub columns: Vec<TableColumn>,
}

impl DatabaseTableDefinition {
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: Identifier::new(table_name).unwrap(), // TODO: Clean this up
            columns: Vec::new(),
        }
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
        self.columns.push(column.into());
        self
    }
}
