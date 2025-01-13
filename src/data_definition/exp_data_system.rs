use std::{any::TypeId, cell::RefCell, collections::HashMap, sync::Arc};

use sqlx::{Postgres, QueryBuilder};

use crate::{
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
        let resources = resources
            .into_iter()
            .map(|(k, v)| (k, RefCell::new(v)))
            .collect::<HashMap<_, _>>();

        // Make sure all child tables are appropriately added.
        // let mut children_of_tables = Vec::new();

        // First pass: Make sure all child tables exist, and modify them where needed.
        for (_, table_def) in &resources {
            let table_def = table_def.borrow();
            let _table_name = table_def.table_name();
            let _table_id_col = table_def.columns().get(&Identifier::new_unchecked("id")).unwrap();
            for _child_table in table_def.child_tables() {
                // match child_table {
                //     super::table::TableRelationship::OneToMany(child_name) => {
                //         let child_table_type_id = table_name_to_type
                //             .get(&child_name)
                //             .expect(&format!(
                //                 "Expected child table {child_name} does not exist in Data System. Aborting."
                //             ));
                //         let mut child_table = resources.get(&child_table_type_id)
                //             .expect(&format!(
                //                 "Expected child table {child_name} does not exist in Data System. Aborting."
                //             )).borrow_mut();
                //         // TODO (UUID id requirement): Make this more dynamic when I remove the "must have UUID" requirement
                //         let parent_table_col_name = format!("{table_name}_id");
                //         child_table.add_column(
                //             TableColumn::new_uuid(&parent_table_col_name)?
                //                 .non_null()
                //                 .fk_to(table_name.clone(), table_id_col.clone()),
                //         );
                //     },
                //     super::table::TableRelationship::ManyToMany(_child_name) => {
                //         todo!("Need to create a NEW join table connecting these")
                //     },
                //     super::table::TableRelationship::OneToOne(child_name) => {
                //         let child_table = table_name_to_type
                //             .get(&child_name)
                //             .and_then(|type_id| resources.get(type_id));
                //         // Just make sure the child table exists - migrations will handle the actual column name down the line.
                //         // TODO: Actually migrations don't do it, it happens in the macro. I should move it here for posterity's sake. Can happen later.
                //         assert!(child_table.is_some());
                //     },
                // }
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
