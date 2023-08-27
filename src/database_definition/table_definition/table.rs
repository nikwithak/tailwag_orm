use super::{DatabaseColumnType, Identifier, TableColumn};

// The details of the Database table. Used to generate the queries for setting up and iteracting with the database.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DatabaseTableDefinition {
    pub table_name: Identifier,
    // TODO: Make it so that there can only be one ID column.
    // TODO: Compoisite keys, Constraints, etc.
    pub columns: Vec<TableColumn>,
}

impl DatabaseTableDefinition {
    pub fn new(table_name: String) -> Self {
        Self {
            table_name: Identifier::new(table_name).unwrap(), // TODO: Clean this up
            columns: Vec::new(),
        }
    }

    pub fn add_column(
        mut self,
        column: TableColumn,
    ) -> Self {
        self.columns.push(column);
        self
    }

    fn add_column_helper(
        &mut self,
        column_name: String,
        column_type: DatabaseColumnType,
    ) {
        self.columns.push(TableColumn::new(
            Identifier::new(column_name).unwrap(),
            column_type,
            Vec::new(),
        ));
    }

    // TODO: Audit Trail stuff. Create/modify/audit-tables, or a list of audit
    // trail associated to the table.
    // fn standard_audit_trail() -> Self {}

    pub fn add_column_int(
        mut self,
        column_name: String,
    ) -> Self {
        self.add_column_helper(column_name, DatabaseColumnType::Int);
        self
    }

    pub fn add_column_bool(
        mut self,
        column_name: String,
    ) -> Self {
        self.add_column_helper(column_name, DatabaseColumnType::Boolean);
        self
    }

    pub fn add_column_float(
        mut self,
        column_name: String,
    ) -> Self {
        self.add_column_helper(column_name, DatabaseColumnType::Float);
        self
    }

    pub fn add_column_string(
        mut self,
        column_name: String,
    ) -> Self {
        self.add_column_helper(column_name, DatabaseColumnType::String);
        self
    }

    pub fn add_column_timestamp(
        mut self,
        column_name: String,
    ) -> Self {
        self.add_column_helper(column_name, DatabaseColumnType::Timestamp);
        self
    }

    pub fn add_column_uuid(
        mut self,
        column_name: String,
    ) -> Self {
        self.columns.push(TableColumn::new(
            Identifier::new(column_name).unwrap(),
            super::DatabaseColumnType::Uuid,
            Vec::new(),
        ));
        self
    }
}
