use reqwest::{self};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use uuid::Uuid;

use super::traits::DataProvider;

/// Creates a DataProvider that will fetch the given type from the provided REST endpoint.
/// Makes specific assumptions about the way the REST endpoint works, and implements this for the base-case
#[derive(Clone)]
pub struct RestApiDataProvider<T>
where
    T: Serialize + for<'d> Deserialize<'d>,
{
    endpoint: String,
    http_client: reqwest::Client,
    _t: PhantomData<T>,
}

impl<T> RestApiDataProvider<T>
where
    T: Serialize + for<'d> Deserialize<'d>,
{
    pub fn from_endpoint(url: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            endpoint: url,
            http_client: client,
            _t: PhantomData,
        }
    }
}

pub trait Id {
    fn id(&self) -> &Uuid;
}

impl<T> DataProvider<T> for RestApiDataProvider<T>
where
    T: Serialize + for<'d> Deserialize<'d> + Id + Default + Clone + Send + Sync,
{
    // TODO: Figure out how to map CreateRequest
    type CreateRequest = T;
    type Error = crate::Error;

    async fn all(&self) -> Result<impl Iterator<Item = T>, Self::Error> {
        Ok(self
            .http_client
            .get(&self.endpoint)
            .send()
            .await
            .unwrap()
            .json::<Vec<T>>()
            .await
            .unwrap()
            .into_iter())
    }

    async fn get(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<T>, Self::Error> {
        let url = format!("{}/{}", &self.endpoint, &id);
        let response = self.http_client.get(&url).send().await.unwrap();
        // if response.status() == StatusCode::NOT_FOUND {
        //     None
        // } else {
        // }
        Ok(response.json::<Option<T>>().await?)
    }

    async fn create(
        &self,
        item: Self::CreateRequest,
    ) -> Result<T, Self::Error> {
        let _response = self.http_client.post(&self.endpoint).json(&item).send().await?;
        Ok(item)
    }

    async fn delete(
        &self,
        item: T,
    ) -> Result<(), Self::Error> {
        // let url = format!("{}/{}", &self.endpoint, &item.id());
        let url = &self.endpoint;
        let _response = self.http_client.delete(url).json(&item).send().await.unwrap();
        Ok(())
    }

    async fn update(
        &self,
        item: &T,
    ) -> Result<(), Self::Error> {
        // let url = format!("{}/{}", &self.endpoint, &item.id());
        let url = &self.endpoint;
        let _response = self.http_client.patch(url).json(item).send().await.unwrap();
        Ok(())
    }
}
