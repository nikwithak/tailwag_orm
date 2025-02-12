use std::{any::TypeId, cell::RefCell, collections::HashMap, sync::Arc};

use sqlx::{Postgres, QueryBuilder};

use crate::{
    data_definition::table::{self, DatabaseColumnType, TableColumn},
    data_manager::{GetTableDefinition, PostgresDataProvider},
    migration::Migration,
    queries::Insertable,
    BuildSql,
};

use super::table::{raw_data::TableDefinition, DatabaseTableDefinition, Identifier};

// pub(crate) trait GenericizedTableDefinition: std::any::Any + TableDefinition {}
pub(crate) type TableDef = Arc<DatabaseTableDefinition>;

// impl GenericizedTableDefinition for DatabaseTableDefinition {}

#[derive(Default)]
pub struct DataSystemBuilder {
    resources: HashMap<TypeId, DatabaseTableDefinition>,
    table_name_to_type: HashMap<Identifier, TypeId>,
}

impl DataSystemBuilder {
    pub fn add_resource<T: GetTableDefinition + Send + 'static>(&mut self) {
        // TODO: On get_table_definition, need to include child tables too and add them here.
        // Fixes the issue where I have to add endpoints for child resources even if I don't want them.
        let table_def = T::get_table_definition();
        self.add_table_def::<T>(table_def);
    }
    pub fn with_resource<T: GetTableDefinition + Send + 'static>(mut self) -> Self {
        self.add_resource::<T>();
        self
    }

    pub(crate) fn add_table_def<T: Send + 'static>(
        &mut self,
        table_def: DatabaseTableDefinition,
    ) {
        let type_id = TypeId::of::<T>();
        self.table_name_to_type.insert(table_def.table_name.clone(), type_id);
        self.resources.insert(type_id, table_def);
    }
    // pub fn get<T: GetTableDefinition + Clone + Send + 'static>(
    //     &self
    // ) -> Option<DatabaseTableDefinition> {
    //     self.resources.get(&TypeId::of::<T>()).map(|t| {
    //         let boxed = <dyn Any>::downcast_ref::<DatabaseTableDefinition>(t).expect(
    //             "Invalid type stored in DataSystem.resources - this should not be possible.
    //             The type exists in the map but failed to downcast.",
    //         );
    //         boxed.clone()

    // }

    pub fn build(self) -> Result<UnconnectedDataSystem, crate::Error> {
        let Self {
            mut resources,
            ..
        } = self;

        let mut stack = resources.clone().into_iter().collect::<Vec<_>>();

        while let Some((_type_id, table_def)) = stack.pop() {
            for (child_type_id, child_def) in table_def.child_tables().into_iter() {
                stack.push((child_type_id, *child_def.clone()));
                resources.insert(child_type_id, *child_def);
            }
        }

        // Put things into a RefCell sot hat we can modify during iteration.
        let mut resources = resources
            .into_iter()
            .map(|(k, v)| (k, RefCell::new(v)))
            .collect::<HashMap<_, _>>();

        // Make sure all child tables are appropriately added.
        // let mut children_of_tables = Vec::new();

        // First pass: Make sure all child tables exist, and modify them where needed.
        let mut new_tables = Vec::new();
        for (parent_type_id, table_def) in &resources {
            let mut table_def = table_def.borrow_mut();
            let table_name = table_def.table_name();
            let table_id_col =
                table_def.columns().get(&Identifier::new_unchecked("id")).unwrap().clone();
            let columns = table_def.columns().clone();
            // For each column - modify its child table appropriately, if exists.
            for (col_name, col_def) in columns {
                let col_type = &col_def.column_type;
                type E = DatabaseColumnType;
                match col_type {
                    E::OneToMany(child_name, table) => {
                        let mut child_table= self.table_name_to_type
                            .get(&child_name)
                            .and_then(|type_id|resources.get(&type_id))
                            .expect(&format!(
                                "Expected child table {child_name} does not exist in Data System. Aborting."
                            )).borrow_mut();
                        // TODO (UUID id requirement): Make this more dynamic when I remove the "must have UUID" requirement
                        let parent_table_col_name = format!("{table_name}_id");
                        child_table.add_column(
                            TableColumn::new_uuid(&parent_table_col_name)?
                                .non_null()
                                .fk_to(table_name.clone(), table_id_col.clone()),
                        );
                    },
                    E::ManyToMany(child_name, _) => {
                        let child_table = self
                            .table_name_to_type
                            .get(&child_name)
                            .and_then(|type_id| resources.get(type_id))
                            .expect(&format!("Expected child table {child_name} to exist"))
                            .borrow();

                        let parent_pk = table_def.get_primary_key().expect("PK should exist");
                        let child_pk = child_table.get_primary_key().expect("PK should exist");

                        let join_table = DatabaseTableDefinition::new(&format!(
                            "{}__to__{}",
                            &table_name, &child_name
                        ))?
                        // TODO: This doesn't *have* to be UUID
                        .with_uuid(&format!("{}_{}", &table_name, &parent_pk.column_name))?
                        .with_uuid(&format!("{}_{}", &child_name, &child_pk.column_name))?;

                        new_tables.push(join_table);
                    },
                    E::OneToOne(child_name, _) => {
                        let child_table = self
                            .table_name_to_type
                            .get(&child_name)
                            .and_then(|type_id| resources.get(type_id))
                            .expect(&format!("Expected child table {child_name} to exist"));
                        let child_pk = child_table
                            .borrow()
                            .get_primary_key()
                            .expect("Child table does not have a primary key to associate to.");
                        // Just make sure the child table exists - migrations will handle the actual column name down the line.
                        // TODO: Actually migrations don't do it, it happens in the macro. I should move it here for posterity's sake. Can happen later.
                        let new_col_def = (*col_def)
                            .clone()
                            .fk_to(child_table.borrow().table_name.clone(), child_pk);
                        // Replace the column in the table definition
                        table_def.add_column(new_col_def);
                    },
                    _ => (), // Nothing to preprocess
                }
            }
        }
        // Add the join tables to resources map. Since we won't ever use these directly, we can create some dummy types to fill the map.

        struct DummyTypeId;
        struct DummyTypeId2;
        struct DummyTypeId3;
        struct DummyTypeId4;
        struct DummyTypeId5;
        // macro_rules! create_dummy_type_ids {
        //     ($name:ty,) => {
        //         TypeId::of::<$name>,
        //     };
        //     ($first_name:ty, $(name:ty,)*) => {
        //         TypeId::of::<($first_name, $($name,)*)>,
        //     };
        // }
        // TODO: If I do tuples, this only needs to be one type after all.
        let mut dummy_type_ids = vec![
            TypeId::of::<DummyTypeId>(),
            TypeId::of::<DummyTypeId2>(),
            TypeId::of::<DummyTypeId3>(),
            TypeId::of::<DummyTypeId4>(),
            TypeId::of::<DummyTypeId5>(),
            TypeId::of::<(DummyTypeId, DummyTypeId)>(),
            TypeId::of::<(DummyTypeId, DummyTypeId2)>(),
            TypeId::of::<(DummyTypeId, DummyTypeId3)>(),
            TypeId::of::<(DummyTypeId, DummyTypeId4)>(),
            TypeId::of::<(DummyTypeId, DummyTypeId5)>(),
            TypeId::of::<(DummyTypeId2, DummyTypeId)>(),
            TypeId::of::<(DummyTypeId2, DummyTypeId2)>(),
            TypeId::of::<(DummyTypeId2, DummyTypeId3)>(),
            TypeId::of::<(DummyTypeId2, DummyTypeId4)>(),
            TypeId::of::<(DummyTypeId2, DummyTypeId5)>(),
        ];
        for new_table in new_tables {
            resources.insert(
                dummy_type_ids
                    .pop()
                    .expect("Must modify the code to support more than 15 join tables"),
                RefCell::new(new_table),
            );
        }

        // TODO: One last pass, to make sure all the child tables are up to date after the changes.
        // On second thought, I don't think this solves the problem. Need to traverse the whole DB tree bottom up to really make sure.
        for (parent_type_id, table_def) in &resources {
            let mut table_def = table_def.borrow_mut();
            for (_, child_col) in table_def.columns().clone() {
                match &child_col.column_type {
                    DatabaseColumnType::OneToMany(identifier, database_table_definition) => {
                        let mut new_col = (*child_col).clone();

                        let child_table = self
                            .table_name_to_type
                            .get(identifier)
                            .and_then(|i| resources.get(i))
                            .expect("Child table missing");
                        new_col.column_type = DatabaseColumnType::OneToMany(
                            identifier.clone(),
                            child_table.borrow().clone(),
                        );
                    },
                    DatabaseColumnType::ManyToMany(identifier, database_table_definition) => {
                        let mut new_col = (*child_col).clone();

                        let child_table = self
                            .table_name_to_type
                            .get(identifier)
                            .and_then(|i| resources.get(i))
                            .expect("Child table missing");
                        new_col.column_type = DatabaseColumnType::ManyToMany(
                            identifier.clone(),
                            child_table.borrow().clone(),
                        );
                    },
                    DatabaseColumnType::OneToOne(identifier, database_table_definition) => {
                        let mut new_col = (*child_col).clone();

                        let child_table = self
                            .table_name_to_type
                            .get(identifier)
                            .and_then(|i| resources.get(i))
                            .expect("Child table missing");
                        new_col.column_type = DatabaseColumnType::OneToOne(
                            identifier.clone(),
                            child_table.borrow().clone(),
                        );
                    },
                    _ => (),
                }
            }
        }

        Ok(UnconnectedDataSystem {
            resources: Arc::new(
                resources.into_iter().map(|(k, v)| (k, Arc::new(v.into_inner()))).collect(),
            ),
        })
    }
}

#[derive(Clone)]
pub struct UnconnectedDataSystem {
    resources: Arc<HashMap<TypeId, Arc<DatabaseTableDefinition>>>,
}
impl UnconnectedDataSystem {
    pub async fn connect(
        &self,
        pool: sqlx::Pool<Postgres>,
    ) -> DataSystem {
        DataSystem {
            resources: self.resources.clone(),
            pool,
        }
    }
}

#[derive(Clone)]
pub struct DataSystem {
    resources: Arc<HashMap<TypeId, Arc<DatabaseTableDefinition>>>,
    pool: sqlx::Pool<Postgres>,
}

impl DataSystem {
    pub fn builder() -> DataSystemBuilder {
        DataSystemBuilder::default()
    }

    pub(crate) fn get_table_def(
        &self,
        type_id: &TypeId,
    ) -> Option<Arc<DatabaseTableDefinition>> {
        self.resources.get(type_id).cloned()
    }
}

impl DataSystem {
    pub fn get<T: Clone + Insertable + Send + 'static>(&self) -> Option<PostgresDataProvider<T>> {
        self.resources
            .get(&TypeId::of::<T>())
            .map(|t| PostgresDataProvider::new(t.clone(), self.pool.clone()))
    }

    pub async fn run_migrations(&self) -> Result<(), crate::Error> {
        fn get_prev_tables_if_exists() -> Option<Vec<Arc<DatabaseTableDefinition>>> {
            std::fs::read(".table_data/last.migration")
                .ok()
                .and_then(|bytes| serde_json::from_slice(bytes.as_slice()).ok())
        }

        fn _save_prev_tables(
            database: Vec<Arc<DatabaseTableDefinition>>
        ) -> Result<(), std::io::Error> {
            let deser = serde_json::to_string(&database)?;
            let bytes = deser.as_bytes();

            // Currently panicing - failing to serialize.
            std::fs::write(".table_data/last.migration", bytes)?;
            Ok(())
        }

        let current_config: Vec<Arc<DatabaseTableDefinition>> =
            self.resources.values().map(|table| table.to_owned()).collect();
        if let Some(migrations) =
            Migration::compare(get_prev_tables_if_exists(), current_config.clone())
        {
            let mut transaction = self.pool.begin().await?;
            for action in migrations.actions {
                let mut builder = QueryBuilder::new("");
                action.build_sql(&mut builder);
                builder.build().execute(&mut *transaction).await?;
            }
            transaction.commit().await?;
        }
        // TODO: It's crashing when trying to serialze the migrations. Need to dig in.
        // save_prev_tables(current_config)?;
        Ok(())
    }
}

impl<T> TryFrom<&DataSystem> for PostgresDataProvider<T>
where
    T: Insertable + Clone + Send + 'static,
{
    type Error = String;

    fn try_from(parent: &DataSystem) -> Result<Self, Self::Error> {
        let Some(val) = parent.get::<T>() else {
            return Err("Unable to fetch PostgresDataProvider form DataSystem".to_string());
        };
        Ok(val)
    }
}
