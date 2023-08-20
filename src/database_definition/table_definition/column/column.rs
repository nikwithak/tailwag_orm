use crate::AsSql;

use crate::database_definition::table_definition::Identifier;

use super::{TableColumnConstraint, TableColumnConstraintDetail};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DatabaseColumnType {
    Boolean,   // BOOL or BOOLEAN
    Int,       // INT
    Float,     // FLOAT
    String,    // VARCHAR or TEXT
    Timestamp, // TIMESTAMP
    Uuid,      // UUID
}

impl DatabaseColumnType {
    pub fn as_str(&self) -> &str {
        match self {
            DatabaseColumnType::Boolean => "BOOL",
            DatabaseColumnType::Int => "INT",
            DatabaseColumnType::Float => "FLOAT",
            DatabaseColumnType::String => "VARCHAR",
            DatabaseColumnType::Timestamp => "TIMESTAMP",
            DatabaseColumnType::Uuid => "UUID",
        }
    }
}

/// Represents a table column. Primarily-based on the PostgreSQL spec for table definition
/// The goal is to get full parity with PostgreSQL.
///
/// [ref](https://www.postgresql.org/docs/current/sql-createtable.html)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TableColumn {
    pub parent_table_name: Identifier,
    pub column_name: Identifier,         // Part of spec
    pub column_type: DatabaseColumnType, // Part of spec - named `data_type in spec
    // pub compression_method: Optional<CompressionMethod>, // TODO: Adds `COMPRESSION compression_method` after `data_type`
    // pub collation: Optional<Collation>, // TODO: Adds `COMPRESSION compression_method` after `data_type`
    pub constraints: Vec<TableColumnConstraint>,
}

impl AsSql for TableColumn {
    fn as_sql(&self) -> String {
        let mut statement = format!("{} {}", &self.column_name, &self.column_type.as_str());

        let mut constraints_iter = self.constraints.iter();
        while let Some(constraint) = constraints_iter.next() {
            statement.push_str(" ");
            statement.push_str(&constraint.as_sql());
        }
        statement
    }
}
