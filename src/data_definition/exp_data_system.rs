use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use sqlx::Postgres;

use crate::{
    data_manager::{GetTableDefinition, PostgresDataProvider},
    queries::Insertable,
};

use super::table::DatabaseTableDefinition;

#[derive(Default)]
pub struct DataSystemBuilder {
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl DataSystemBuilder {
    pub fn add_resource<T: GetTableDefinition + 'static>(&mut self) {
        self.resources.insert(TypeId::of::<T>(), Box::new(T::get_table_definition()));
    }
    pub fn with_resource<T: GetTableDefinition + 'static>(mut self) -> Self {
        self.add_resource::<T>();
        self
    }

    pub fn build(self) -> UnconnectedDataSystem {
        UnconnectedDataSystem {
            resources: Arc::new(self.resources),
        }
    }
}

pub struct UnconnectedDataSystem {
    resources: Arc<HashMap<TypeId, Box<dyn Any>>>,
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
    resources: Arc<HashMap<TypeId, Box<dyn Any>>>,
    pool: sqlx::Pool<Postgres>,
}

impl DataSystem {
    pub fn builder() -> DataSystemBuilder {
        DataSystemBuilder::default()
    }
}

impl DataSystem {
    pub fn get<T: Clone + Insertable + 'static>(&self) -> Option<PostgresDataProvider<T>> {
        self.resources.get(&TypeId::of::<T>()).map(|t| {
            let boxed = t.downcast_ref::<DatabaseTableDefinition<T>>().expect(
                "Invalid type stored in DataSystem.resources - this should not be possible.
                The type exists in the map but failed to downcast.",
            );
            PostgresDataProvider::new(boxed.clone(), self.pool.clone())
        })
    }
}

impl<T> TryFrom<&DataSystem> for PostgresDataProvider<T>
where
    T: Insertable + Clone + 'static,
{
    type Error = String;

    fn try_from(parent: &DataSystem) -> Result<Self, Self::Error> {
        let Some(val) = parent.get::<T>() else {
            return Err("Unable to fetch PostgresDataProvider form DataSystem".to_string());
        };
        Ok(val)
    }
}
