use crate::data_manager::{rest_api::Id, traits::DataProvider};
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

impl<T: Clone + Id + Send + Default + Sync> DataProvider<T> for InMemoryDataProvider<T> {
    type CreateRequest = T;
    type Error = crate::Error;

    async fn all(&self) -> Result<impl Iterator<Item = T>, Self::Error> {
        // todo!();
        // Ok(vec![].into_iter())
        let items = self.items.lock().map_err(|e| crate::Error::MutexLock(e.to_string()))?;
        Ok(items.values().map(|v| v.to_owned()).collect::<Vec<_>>().into_iter())
    }

    async fn get(
        &self,
        id: Uuid,
    ) -> Result<Option<T>, Self::Error> {
        let items = self.items.lock().unwrap();
        let item = items.get(&id).cloned();
        Ok(item)
    }

    async fn create(
        &self,
        item: Self::CreateRequest,
    ) -> Result<T, Self::Error> {
        let mut items = self.items.lock().unwrap();
        let id = item.id();
        if items.contains_key(id) {
            Err(format!("Already contains object with id ({})", id))?
        } else {
            items.insert(*item.id(), item.clone());
            Ok(item)
        }
    }

    async fn delete(
        &self,
        item: T,
    ) -> Result<(), Self::Error> {
        let mut items = self.items.lock().unwrap();
        match items.remove(item.id()) {
            Some(_) => Ok(()),
            None => Err("Item not found".to_string())?,
        }
    }

    async fn update(
        &self,
        item: &T,
    ) -> Result<(), Self::Error> {
        let mut items = self.items.lock().unwrap();
        let id = item.id();
        if items.contains_key(id) {
            items.insert(*item.id(), item.clone());
            Ok(())
        } else {
            Err(format!("no object with id ({})", id))?
        }
    }
}
