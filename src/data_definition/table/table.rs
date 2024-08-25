use std::{collections::BTreeMap, marker::PhantomData, ops::Deref, sync::Arc};

use super::{Identifier, TableColumn, TableConstraint};

// The details of the Database table. Used to generate the queries for setting up and iteracting with the database.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DatabaseTableDefinition<T> {
    data: Arc<DatabaseTableDefinitionData<T>>,
}

impl<T> Deref for DatabaseTableDefinition<T> {
    type Target = DatabaseTableDefinitionData<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DatabaseTableDefinition<T> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(table_name: &str) -> Result<DatabaseTableDefinitionData<T>, String> {
        DatabaseTableDefinitionData::<T>::new(table_name)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TableRelationship {
    OneToMany(Identifier),
    ManyToMany(Identifier),
    OneToOne(Identifier),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DatabaseTableDefinitionData<T> {
    pub table_name: Identifier,
    // TODO: Make it so that there can only be one ID column.
    // TODO: Composite keys, Constraints, etc.
    // pub columns: Vec<TableColumn>,
    pub columns: BTreeMap<Identifier, TableColumn>, // BTreeMap for testing reasons... yes it adds inefficiency, but shoudln't be enough to matter.
    pub child_tables: Vec<TableRelationship>,       // local_name to table_name
    pub constraints: Vec<TableConstraint>,
    pub _t: PhantomData<T>,
}

impl<T> From<DatabaseTableDefinitionData<T>> for DatabaseTableDefinition<T> {
    fn from(val: DatabaseTableDefinitionData<T>) -> Self {
        DatabaseTableDefinition {
            data: Arc::new(val),
        }
    }
}

impl<T> DatabaseTableDefinitionData<T> {
    pub fn new(table_name: &str) -> Result<Self, String> {
        Ok(Self {
            table_name: Identifier::new(table_name)?, // TODO: Clean this up ([2023-12-11] What's wrong with it / clean up in what way?)
            columns: BTreeMap::new(),
            child_tables: Default::default(),
            // columns: Vec::new(),
            constraints: Vec::new(),
            _t: PhantomData,
        })
    }

    pub fn column<C: Into<TableColumn>>(
        mut self,
        column: C,
    ) -> Self {
        self.add_column(column);
        self
    }

    pub fn add_column<C: Into<TableColumn>>(
        &mut self,
        column: C,
    ) {
        let column = column.into();
        self.columns.insert(column.column_name.clone(), column);
        // self.columns.push(column);
    }
}

impl<T> DatabaseTableDefinitionData<T> {
    // /// Creates a one-to-one relationship between the two objects. This will add a Foriegn Key column
    // /// for each column making up the downstream table's Primary Key.
    // TODO: Might change how this is done. Delete if not needed later.
    // pub fn with_one_to_one(
    //     self,
    //     col_name: &str,
    //     ref_table: &DatabaseTableDefinition<T>
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
