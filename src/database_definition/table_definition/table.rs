use super::{Identifier, TableColumn};

// The details of the Database table. Used to generate the queries for setting up and iteracting with the database.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DatabaseTableDefinition {
    pub table_name: Identifier,
    // TODO: Make it so that there can only be one ID column.
    // TODO: Compoisite keys, Constraints, etc.
    pub columns: Vec<TableColumn>,
}
