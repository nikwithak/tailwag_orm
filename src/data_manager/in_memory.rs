use crate::data_manager::{
    rest_api::Id,
    traits::{DataProvider, DataResult},
};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

/// InMemoryDataProvider - Wraps a HashMap in the DatProvider interface.
/// Status: Prototype / Hacky
/// Notes: Lots of data duplication, not memory efficient, and blocks on reads due to the Arc<Mutex>, which is required to fit the trait signtautres and enable Clone

#[derive(Clone, Default)]
pub struct InMemoryDataProvider<T> {
    items: Arc<Mutex<HashMap<Uuid, T>>>,
}

#[async_trait]
impl<T: Clone + Id + Send + Default + Sync> DataProvider<T> for InMemoryDataProvider<T> {
    type CreateRequest = T;
    type QueryType = Vec<T>;

    async fn all(&self) -> DataResult<Self::QueryType> {
        let items = self.items.lock().unwrap();
        Ok(items.values().map(|item| item.clone()).collect())
    }

    async fn get(
        &self,
        id: Uuid,
    ) -> DataResult<Option<T>> {
        let items = self.items.lock().unwrap();
        let item = items.get(&id).map(|item| item.clone());
        Ok(item)
    }

    async fn create(
        &self,
        item: Self::CreateRequest,
    ) -> Result<T, String> {
        let mut items = self.items.lock().unwrap();
        let id = item.id();
        if items.contains_key(id) {
            Err(format!("Already contains object with id ({})", id))
        } else {
            items.insert(item.id().clone(), item.clone());
            Ok(item)
        }
    }

    async fn delete(
        &self,
        item: T,
    ) -> Result<(), String> {
        let mut items = self.items.lock().unwrap();
        match items.remove(item.id()) {
            Some(_) => Ok(()),
            None => Err("Item not found".to_string()),
        }
    }

    async fn update(
        &self,
        item: &T,
    ) -> Result<(), String> {
        let mut items = self.items.lock().unwrap();
        let id = item.id();
        if items.contains_key(id) {
            items.insert(item.id().clone(), item.clone());
            Ok(())
        } else {
            Err(format!("no object with id ({})", id))
        }
    }
}
