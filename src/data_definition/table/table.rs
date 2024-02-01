use std::{
    collections::{BTreeMap},
    ops::Deref,
    sync::Arc,
};

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
pub enum TableRelationship {
    OneToMany(Identifier),
    ManyToMany(Identifier),
    OneToOne(Identifier),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DatabaseTableDefinitionData {
    pub table_name: Identifier,
    // TODO: Make it so that there can only be one ID column.
    // TODO: Composite keys, Constraints, etc.
    // pub columns: Vec<TableColumn>,
    pub columns: BTreeMap<Identifier, TableColumn>, // BTreeMap for testing reasons... yes it adds inefficiency, but shoudln't be enough to matter.
    pub child_tables: Vec<TableRelationship>,       // local_name to table_name
    pub constraints: Vec<TableConstraint>,
}

impl From<DatabaseTableDefinitionData> for DatabaseTableDefinition {
    fn from(val: DatabaseTableDefinitionData) -> Self {
        DatabaseTableDefinition {
            data: Arc::new(val),
        }
    }
}

impl DatabaseTableDefinitionData {
    pub fn new(table_name: &str) -> Result<Self, String> {
        Ok(Self {
            table_name: Identifier::new(table_name)?, // TODO: Clean this up ([2023-12-11] What's wrong with it / clean up in what way?)
            columns: BTreeMap::new(),
            child_tables: Default::default(),
            // columns: Vec::new(),
            constraints: Vec::new(),
        })
    }

    pub fn column<T: Into<TableColumn>>(
        mut self,
        column: T,
    ) -> Self {
        self.add_column(column);
        self
    }

    pub fn add_column<T: Into<TableColumn>>(
        &mut self,
        column: T,
    ) {
        let column = column.into();
        self.columns.insert(column.column_name.clone(), column);
        // self.columns.push(column);
    }
}

impl DatabaseTableDefinitionData {
    // /// Creates a one-to-one relationship between the two objects. This will add a Foriegn Key column
    // /// for each column making up the downstream table's Primary Key.
    // TODO: Might change how this is done. Delete if not needed later.
    // pub fn with_one_to_one(
    //     self,
    //     col_name: &str,
    //     ref_table: &DatabaseTableDefinition,
    // ) -> Result<Self, String> {
    //     let t: Vec<_> = ref_table
    //         .data
    //         .constraints
    //         .iter()
    //         .filter_map(|constraint| {
    //             if let TableConstraintDetail::PrimaryKey(pk) = constraint.detail.as_ref() {
    //                 Some(pk)
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect();
    //     for fk in t {
    //         for ref_col in &fk.columns {
    //             let column_name = format!("{}_{}", col_name, &ref_col.column_name);
    //             TableColumn::new(&column_name, ref_col.column_type.clone(), Vec::new())?
    //                 .fk_to(ref_table.clone(), ref_col.clone());
    //         }
    //     }
    //     Ok(self)
    // }

    pub fn with_string(
        self,
        col_name: &str,
    ) -> Result<Self, String> {
        Ok(self.column(TableColumn::string(col_name)?))
    }
    pub fn with_bool(
        self,
        col_name: &str,
    ) -> Result<Self, String> {
        Ok(self.column(TableColumn::bool(col_name)?))
    }
    pub fn with_float(
        self,
        col_name: &str,
    ) -> Result<Self, String> {
        Ok(self.column(TableColumn::float(col_name)?))
    }
    pub fn with_int(
        self,
        col_name: &str,
    ) -> Result<Self, String> {
        Ok(self.column(TableColumn::int(col_name)?))
    }
    pub fn with_timestamp(
        self,
        col_name: &str,
    ) -> Result<Self, String> {
        Ok(self.column(TableColumn::timestamp(col_name)?))
    }
    pub fn with_uuid(
        self,
        col_name: &str,
    ) -> Result<Self, String> {
        Ok(self.column(TableColumn::uuid(col_name)?))
    }
}
