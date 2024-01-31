use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{data_definition::table::DatabaseTableDefinition, data_manager::traits::DataProvider};
use std::{fs, marker::PhantomData, path::PathBuf};

use super::{rest_api::Id, traits::DataResult};

/// UNTESTED - USE AT YOUR OWN RISK
#[derive(Clone)]
pub struct LocalFileDataProvider<T> {
    pub table_definition: DatabaseTableDefinition,
    pub root_folder_path: PathBuf,
    pub _t: PhantomData<T>,
}

impl<T: Id> LocalFileDataProvider<T> {
    fn get_filepath(
        &self,
        item_id: &Uuid,
    ) -> PathBuf {
        let filename = format!("{}.json", item_id);
        let mut path = self.root_folder_path.clone();
        path.push(&filename);
        path.to_owned()
    }
}

#[async_trait]
impl<T: Default + Sync + Send + Id + Serialize + for<'a> Deserialize<'a>> DataProvider<T>
    for LocalFileDataProvider<T>
{
    type CreateRequest = T;
    type QueryType = Vec<T>;

    async fn all(&self) -> DataResult<Self::QueryType> {
        todo!()
    }

    async fn create(
        &self,
        item: Self::CreateRequest,
    ) -> Result<T, String> {
        // let item: T = item.into();

        let path = self.get_filepath(item.id());
        let contents = serde_json::to_string(&item).unwrap();
        std::fs::write(path, contents).unwrap();

        Ok(item)
    }

    async fn get(
        &self,
        id: Uuid,
    ) -> DataResult<Option<T>> {
        let path = self.get_filepath(&id);
        let contents = std::fs::read_to_string(path).unwrap();
        match serde_json::from_str::<T>(&contents) {
            Ok(a) => Ok(Some(a)),
            Err(e) => Err(format!("{:?}", &e)),
        }
    }

    async fn delete(
        &self,
        item: T,
    ) -> Result<(), String> {
        let path = self.get_filepath(item.id());
        // For safety, make sure it's the right object:
        self.get(*item.id()).await.unwrap();
        match fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{:?}", &e)),
        }
    }

    async fn update(
        &self,
        item: &T,
    ) -> Result<(), String> {
        let path = self.get_filepath(item.id());
        let contents = serde_json::to_string(item).unwrap();
        std::fs::write(path, contents).unwrap();
        Ok(())
    }
}
