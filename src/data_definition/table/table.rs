use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap},
};

use serde::{Deserialize, Serialize};

use super::{Identifier, TableColumn, TableConstraint};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum TableRelationship {
    OneToMany(Identifier),
    ManyToMany(Identifier),
    OneToOne(Identifier),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct DatabaseTableDefinition {
    pub table_name: Identifier,
    // TODO: Make it so that there can only be one ID column.
    // TODO: Composite keys, Constraints, etc.
    // pub columns: Vec<TableColumn>,
    pub columns: BTreeMap<Identifier, TableColumn>, // BTreeMap for testing reasons... yes it adds inefficiency, but shoudln't be enough to matter.
    #[serde(skip)]
    pub child_tables: HashMap<TypeId, Box<DatabaseTableDefinition>>, // Used for auto-adding child tables without explicitly adding them to the Application.
    pub constraints: Vec<TableConstraint>,
}

/// Experimental - we need a typeless vbersion of this data for building migrations appropriately.
/// We could instead remove the PhantomData<T> from `DatabaseTableDefinition` instead, but that's
///   (a) a huge overhaul, and
///   (b) loses some type association data. It would be a huuuuge refactor.
/// (Interestingly, that was the way I *originally* did it, I think. Wish I'd just stuck that way...)
pub(crate) mod raw_data {
    use std::{
        any::TypeId,
        collections::{BTreeMap, HashMap},
    };

    use crate::data_definition::table::{Identifier, TableColumn, TableConstraint};

    use super::DatabaseTableDefinition;
    trait LockedTrait {}
    impl LockedTrait for DatabaseTableDefinition {}

    #[allow(private_bounds, unused)]
    pub trait TableDefinition
    where
        Self: LockedTrait,
    {
        fn table_name(&self) -> Identifier;
        fn child_tables(&self) -> HashMap<TypeId, Box<DatabaseTableDefinition>>;
        fn constraints(&self) -> &Vec<TableConstraint>;
        fn columns(&self) -> &BTreeMap<Identifier, TableColumn>;
        fn add_column(
            &mut self,
            column: TableColumn,
        );
    }

    impl TableDefinition for DatabaseTableDefinition {
        fn table_name(&self) -> Identifier {
            self.table_name.clone()
        }

        fn constraints(&self) -> &Vec<TableConstraint> {
            &self.constraints
        }

        fn child_tables(&self) -> HashMap<TypeId, Box<DatabaseTableDefinition>> {
            self.child_tables.clone()
        }

        fn columns(&self) -> &BTreeMap<Identifier, TableColumn> {
            &self.columns
        }

        fn add_column(
            &mut self,
            column: TableColumn,
        ) {
            self.add_column(column);
        }
    }
}

impl DatabaseTableDefinition {
    pub fn new(table_name: &str) -> Result<Self, String> {
        Ok(Self {
            table_name: Identifier::new(table_name)?, // TODO: Clean this up ([2023-12-11] What's wrong with it / clean up in what way?)
            columns: BTreeMap::new(),
            child_tables: Default::default(),
            // columns: Vec::new(),
            constraints: Vec::new(),
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

impl DatabaseTableDefinition {
    // /// Creates a one-to-one relationship between the two objects. This will add a Foriegn Key column
    // /// for each column making up the downstream table's Primary Key.
    // TODO: Might change how this is done. Delete if not needed later.
    // pub fn with_one_to_one(
    //     self,
    //     col_name: &str,
    //     ref_table: &DatabaseTableDefinition
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
