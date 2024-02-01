use std::ops::Deref;
use std::sync::Arc;

use crate::AsSql;

use crate::data_definition::table::{DatabaseTableDefinition, Identifier};

use super::{TableColumnConstraint, TableColumnConstraintDetail};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DatabaseColumnType {
    Boolean,   // BOOL or BOOLEAN
    Int,       // INT
    Float,     // FLOAT
    String,    // VARCHAR or TEXT
    Timestamp, // TIMESTAMP
    Uuid,      // UUID
    Json,      // JSONB

    // These next few that define relationship types are a hacky way of building cross-table relationships -
    // I'm dealing with a consequence of deciding that tables would be locked once they were fully built, but this will require that I re-do them down the line.
    OneToMany(Identifier), // TODO: Impl this all the way through
    // ManyToOne(DatabaseTableDefinition),
    ManyToMany(Identifier), // TODO: Will need to figure out how I want to represent JoinTables here
    OneToOne(Identifier), // TODO: With [inline] macro attribute, can make this inline JSON when needed. Depeneds on if we want sortability / searchability or not
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
            DatabaseColumnType::Json => "JSONB",
            DatabaseColumnType::OneToMany(_) => todo!(),
            DatabaseColumnType::ManyToMany {
                ..
            } => todo!(),
            DatabaseColumnType::OneToOne(_) => todo!(),
        }
    }
}

/// Represents a table column. Primarily-based on the PostgreSQL spec for table definition
/// The goal is to get full parity with PostgreSQL.
///
/// [ref](https://www.postgresql.org/docs/current/sql-createtable.html)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableColumn {
    data: Arc<TableColumnData>,
}

impl Deref for TableColumn {
    type Target = TableColumnData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableColumnData {
    // TODO: Is parent_table_name needed here??? Parent should track it, doesn't  need a call back (why did I add this in the first place??)
    // pub parent_table_name: Identifier, // Identifier is just an Arc<String> (with validation) under the hood
    pub column_name: Identifier,         // Part of spec
    pub column_type: DatabaseColumnType, // Part of spec - named `data_type` in spec
    // pub compression_method: Optional<CompressionMethod>, // TODO: Adds `COMPRESSION compression_method` after `data_type`
    // pub collation: Optional<Collation>, // TODO: Adds `COMPRESSION compression_method` after `data_type`
    pub constraints: Vec<TableColumnConstraint>,
}

impl From<TableColumnData> for TableColumn {
    fn from(val: TableColumnData) -> Self {
        TableColumn {
            data: Arc::new(val),
        }
    }
}

impl TableColumnData {
    pub fn is_nullable(&self) -> bool {
        self.constraints
            .iter()
            .find(|constraint| {
                if let TableColumnConstraintDetail::NotNull = &*constraint.detail {
                    true
                } else {
                    false
                }
            })
            .is_none()
    }
    pub fn is_pk(&self) -> bool {
        self.constraints
            .iter()
            .find(|constraint| {
                if let TableColumnConstraintDetail::PrimaryKey(_) = &*constraint.detail {
                    true
                } else {
                    false
                }
            })
            .is_some()
    }
}

impl TableColumnData {
    pub fn non_null(mut self) -> Self {
        self.constraints.push(TableColumnConstraint::non_null());
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.constraints.push(TableColumnConstraint::primary_key());
        self
    }
    pub fn pk(self) -> Self {
        self.primary_key()
    }

    pub fn fk_to(
        self,
        ref_table: DatabaseTableDefinition,
        ref_column: TableColumn,
    ) -> Self {
        self.foreign_key_to(ref_table, ref_column)
    }

    pub fn foreign_key_to(
        mut self,
        ref_table: DatabaseTableDefinition,
        ref_column: TableColumn,
    ) -> Self {
        self.constraints.push(TableColumnConstraint::foreign_key(ref_table, ref_column));
        self
    }
}

impl TableColumn {
    /// Creates a new TableColumn.
    ///
    /// Before using TableColumn::new, see if any of the provided helper
    /// `TableColumn::new_*` functions fit your use case.
    pub fn new(
        column_name: &str,
        column_type: DatabaseColumnType,
        constraints: Vec<TableColumnConstraint>,
    ) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            // parent_table_name,
            column_name: Identifier::new(column_name)?,
            column_type,
            constraints,
        })
    }

    pub fn int(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_int(column_name)
    }
    pub fn timestamp(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_timestamp(column_name)
    }
    pub fn string(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_string(column_name)
    }
    pub fn float(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_float(column_name)
    }
    pub fn uuid(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_uuid(column_name)
    }
    pub fn bool(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_bool(column_name)
    }

    pub fn new_int(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::Int,
            constraints: vec![],
        })
    }

    pub fn new_string(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::String,
            constraints: vec![],
        })
    }

    pub fn new_timestamp(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::Timestamp,
            constraints: vec![],
        })
    }

    pub fn new_float(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::Float,
            constraints: vec![],
        })
    }

    pub fn new_uuid(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::Uuid,
            constraints: vec![],
        })
    }

    pub fn new_bool(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::Boolean,
            constraints: vec![],
        })
    }
}

impl AsSql for TableColumn {
    fn as_sql(&self) -> String {
        let mut statement = format!("{} {}", &self.column_name, &self.column_type.as_str());

        let constraints_iter = self.constraints.iter();
        for constraint in constraints_iter {
            statement.push(' ');
            statement.push_str(&constraint.as_sql());
        }
        statement
    }
}
