use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap},
};

use raw_data::TableDefinition;
use serde::{Deserialize, Serialize};

use crate::queries::{Join, JoinSide, JoinType};

use super::{Identifier, TableColumn, TableConstraint};

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

impl DatabaseTableDefinition {
    pub fn get_primary_key(&self) -> Option<TableColumn> {
        self.columns.iter().map(|(ident, col)| col).find(|col| col.is_pk()).cloned()
    }
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

impl DatabaseTableDefinition {
    pub fn build_select_items(
        &self,
        prefix: &str,
    ) -> Vec<String> {
        type E = crate::data_definition::table::DatabaseColumnType;
        let table_name = &self.table_name;
        self.columns
            .values()
            .filter_map(|col| {
                let col_name = col.column_name.to_string();
                match &col.column_type {
                    E::Boolean
                    | E::Int
                    | E::Float
                    | E::String
                    | E::Timestamp
                    | E::Uuid
                    | E::Json => Some(format!("{prefix}_{table_name}.{col_name}")),
                    E::OneToMany(child_table_ident, child_table_def) => {
                        // TODO
                        // END TODO
                        Some(format!(
                            "COALESCE(json_agg(
                                json_build_object(
                                    {child_table_ident}
                                )
                            ) filter (where {child_table_ident}), '[]')::JSON as {child_table_ident}"
                        ))
                    },
                    E::ManyToMany(_, _todo) => todo!(),
                    // E::OneToOne(_, _todo) => Some(col_name.trim_end_matches("_id").to_string()), // TODO: UNHACK THIS
                    E::OneToOne {
                        col_name,
                        ..
                    } => {
                        // todo!()
                        // Need to loop through ALL columns, and place them deliberately

                        Some(format!("json_agg({})", col_name)) // TODO: UNHACK THIS
                    },
                }
            })
            .peekable()
            .collect()
    }

    pub fn json_build_object(
        &self,
        prefix: &str,
    ) -> String {
        let table_name = &*self.table_name;
        let mut attrs = self
            .columns
            .values()
            .map(|column| {
                let column_name = &*column.column_name;
                match &column.column_type {
                    super::DatabaseColumnType::Boolean
                    | super::DatabaseColumnType::Int
                    | super::DatabaseColumnType::Float
                    | super::DatabaseColumnType::String
                    | super::DatabaseColumnType::Timestamp
                    | super::DatabaseColumnType::Uuid
                    | super::DatabaseColumnType::Json => {
                        // Standard use case - just blop it down
                        format!("'{column_name}', {prefix}{table_name}.{column_name}")
                    },
                    super::DatabaseColumnType::OneToMany(identifier, database_table_definition) => {
                        // Need to coalesce into an array of json_build_objects
                        let child_table_name = &*database_table_definition.table_name;
                        // TODO: Remove the hardcoded .id here
                        format!(
                            "'{identifier}', COALESCE(
                                json_agg(
                                    {}
                                ) filter (
                                    WHERE {prefix}{table_name}_{child_table_name}.id IS NOT NULL
                                ),
                                '[]'
                            )::JSON",
                            database_table_definition
                                .json_build_object(&format!("{prefix}{table_name}_"))
                        )
                    },
                    super::DatabaseColumnType::ManyToMany(
                        identifier,
                        database_table_definition,
                    ) => {
                        // TODO: Look at join table.
                        todo!()
                    },
                    super::DatabaseColumnType::OneToOne {
                        table_def,
                        ..
                    } => {
                        // super::DatabaseColumnType::OneToOne(identifier, database_table_definition) => {
                        let param_name = column_name.strip_suffix("_id").unwrap_or(column_name); // TODO: Store this elsehwere so we don't have to assume ID
                        format!(
                            "'{param_name}', {}",
                            table_def.json_build_object(&format!("{prefix}{table_name}_"))
                        )
                    },
                }
            })
            .peekable();

        let mut ret = String::new();
        ret.push_str("json_build_object(");
        while let Some(attr) = attrs.next() {
            ret.push_str(&attr);
            if attrs.peek().is_some() {
                ret.push_str(", ");
            }
        }
        ret.push_str(")");

        ret
    }

    pub fn get_join_tables(
        &self,
        prefix: &str,
    ) -> Vec<String> {
        let table_alias = format!("{prefix}{}", &*self.table_name);
        let mut results = Vec::new();
        for child_tbl in self.columns.values() {
            match &child_tbl.column_type {
                super::DatabaseColumnType::OneToMany(identifier, database_table_definition) => {
                    let joined_table = &*database_table_definition.table_name;
                    let joined_table_alias = format!("{table_alias}_{joined_table}");
                    // let joined_column = &*child_tbl.column_name; // TODO: NEed to find joined column name instead of assuming "id"
                    let joined_column = format!("{}_id", &*self.table_name);
                    let table_column = &*database_table_definition
                        .get_primary_key()
                        .expect(&format!(
                            "Must have PK to do a join column: {:?}",
                            &database_table_definition
                        ))
                        .column_name;
                    let stmt = format!("LEFT OUTER JOIN {joined_table} {joined_table_alias} ON {joined_table_alias}.{joined_column} = {table_alias}.{table_column}");
                    results.push(stmt);
                    results.append(
                        &mut database_table_definition.get_join_tables(&format!("{table_alias}_")),
                    )
                },
                super::DatabaseColumnType::ManyToMany(identifier, database_table_definition) => {
                    todo!()
                },
                // super::DatabaseColumnType::OneToOne(identifier, database_table_definition) => {
                super::DatabaseColumnType::OneToOne {
                    table_def,
                    ..
                } => {
                    let joined_table = &*table_def.table_name;
                    let joined_table_alias = format!("{table_alias}_{joined_table}");
                    let table_column = &*child_tbl.column_name;
                    let joined_column = &*table_def
                        .get_primary_key()
                        .expect(&format!("Must have PK to do a join column! {:?}", &table_def,))
                        .column_name;
                    let stmt = format!("LEFT OUTER JOIN {joined_table} {joined_table_alias} ON {joined_table_alias}.{joined_column} = {table_alias}.{table_column}");
                    results.push(stmt);
                    results.append(&mut table_def.get_join_tables(&format!("{table_alias}_")))
                },
                _ => (), // No tables to join
            }
        }
        results
    }
}
