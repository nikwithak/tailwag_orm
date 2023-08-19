use crate::AsSql;

use super::Identifier;

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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TableColumn {
    pub parent_table_name: Identifier,
    pub column_name: Identifier,
    pub column_type: DatabaseColumnType,
    // TODO: If this grows too big, add a new type for "ColumnModifiers" or similar
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub foreign_key_to: Option<Box<TableColumn>>,
    // _default: Option<String>, // TODO
}

impl AsSql for TableColumn {
    fn as_sql(&self) -> String {
        #[rustfmt::skip]
        let pk_str = if self.is_primary_key { "PRIMARY KEY" } else { "" };
        #[rustfmt::skip]
        let not_null_str = if self.is_nullable { "" } else { "NOT NULL" };
        format!(
            " {} {} {} {} ",
            &self.column_name,
            &self.column_type.as_str(),
            pk_str,
            not_null_str
        )
    }
}
