use crate::{database_definition::table_definition::DatabaseTableDefinition, AsSql};

use crate::database_definition::table_definition::Identifier;

use super::{
    PrimaryKeyConstraint, ReferencesConstraint, TableColumnConstraint, TableColumnConstraintDetail,
};

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
    // TODO: Is parent_table_name needed here??? Parent should track it, doesn't  need a call back (why did I add this in the first place??)
    // pub parent_table_name: Identifier, // Identifier is just an Arc<String> (with validation) under the hood
    pub column_name: Identifier,         // Part of spec
    pub column_type: DatabaseColumnType, // Part of spec - named `data_type` in spec
    // pub compression_method: Optional<CompressionMethod>, // TODO: Adds `COMPRESSION compression_method` after `data_type`
    // pub collation: Optional<Collation>, // TODO: Adds `COMPRESSION compression_method` after `data_type`
    pub constraints: Vec<TableColumnConstraint>,
}
// TODO: Wrap in an `Arc` with `Deref` - wonder what the idomatic term for this pattern is.

impl TableColumn {
    /// Creates a new TableColumn.
    ///
    /// Before using TableColumn::new, see if any of the provided helper
    /// `TableColumn::new_*` functions fit your use case.
    pub fn new(
        column_name: Identifier,
        column_type: DatabaseColumnType,
        constraints: Vec<TableColumnConstraint>,
    ) -> Self {
        Self {
            // parent_table_name,
            column_name,
            column_type,
            constraints,
        }
    }

    pub fn new_int(column_name: Identifier) -> Self {
        Self {
            column_name,
            column_type: DatabaseColumnType::Int,
            constraints: vec![],
        }
    }

    pub fn new_string(column_name: Identifier) -> Self {
        Self {
            column_name,
            column_type: DatabaseColumnType::String,
            constraints: vec![],
        }
    }

    pub fn new_timestamp(column_name: Identifier) -> Self {
        Self {
            column_name,
            column_type: DatabaseColumnType::Timestamp,
            constraints: vec![],
        }
    }

    pub fn new_float(column_name: Identifier) -> Self {
        Self {
            column_name,
            column_type: DatabaseColumnType::Float,
            constraints: vec![],
        }
    }

    pub fn new_uuid_fk(
        column_name: Identifier,
        target_table: DatabaseTableDefinition,
    ) -> Self {
        // TODO: Don't need to loop once I expose this info from DatabaseTableDefinition
        let foreign_column_name = target_table
            .columns
            .iter()
            .find(|column| {
                // Matches the first column in target_table that is a UUID and a PK
                let is_uuid = column.column_type == DatabaseColumnType::Uuid;
                let is_pk = column
                    .constraints
                    .iter()
                    .find(|constraint| match *constraint.detail {
                        TableColumnConstraintDetail::PrimaryKey(_) => true,
                        _ => false,
                    })
                    .is_some();
                is_uuid && is_pk
            })
            .expect("Uuid foreign key relationship does not exist."); // TODO: Add some error responses instead of just panicking.

        Self {
            column_name,
            column_type: DatabaseColumnType::Float,
            constraints: vec![TableColumnConstraint::foreign_key(
                target_table.table_name.clone(),
                foreign_column_name.column_name.clone(),
            )],
        }
    }

    pub fn new_bool(column_name: Identifier) -> Self {
        Self {
            column_name,
            column_type: DatabaseColumnType::Boolean,
            constraints: vec![],
        }
    }

    pub fn new_uuid_pk(column_name: Identifier) -> Self {
        Self {
            column_name,
            column_type: DatabaseColumnType::Uuid,
            constraints: vec![TableColumnConstraint::primary_key()],
        }
    }
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
