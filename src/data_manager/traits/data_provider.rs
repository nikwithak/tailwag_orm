use std::ops::{Deref, DerefMut};

use async_trait::async_trait;
use uuid::Uuid;

pub type DataResult<T> = Result<T, String>;

/// Provides basic CRUD operations for a type's data source
///
/// Unless you have a specific use case that warrants a custom implementation,
/// it is recommended that you use one of the premade DataProviders for typical
/// use cases (examples below). A long-term goal is to define the configurability
/// of these to handle contexts / user permissions / etc.
///
///  - [x] PostgresDataProvider
///  - [x] TODO: RestApiDataProvider
///  - [x] TODO: InMemoryDataProvider // Intended for debugging/development
///                                   // primarily, but may have other uses (e.g. used in caching)
///  - [ ] TODO: LocalFileDataProvider
///  - [ ] TODO: S3DataProvider
///  - [ ] TODO: CachedDataProvider<P: DataProvider>
///  - { } TODO: PolicyEnforcedDataProvider
///  - [ ] TODO: MultiSourceDataProvider
#[async_trait]
pub trait DataProvider<T> {
    type CreateRequest: Default;
    type QueryType: Into<Vec<T>>;

    // fn all(&self) -> Vec<T>;
    async fn all(&self) -> DataResult<Self::QueryType>;
    async fn get(
        &self,
        id: Uuid,
    ) -> DataResult<Option<T>>;
    async fn create(
        &self,
        item: Self::CreateRequest,
    ) -> DataResult<T>; // TODO: Create real error type when needed
    async fn delete(
        &self,
        item: T, // You give it up when you ask to delete it!
    ) -> DataResult<()>;
    async fn update(
        &self,
        item: &T,
    ) -> DataResult<()>;
}

pub struct Provided<'a, T, P>
where
    P: DataProvider<T>,
{
    provider: &'a P,
    data: T,
}

impl<'a, T, P> DerefMut for Provided<'a, T, P>
where
    P: DataProvider<T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, T, P> Deref for Provided<'a, T, P>
where
    P: DataProvider<T>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[allow(unused)]
impl<'a, T, P> Provided<'a, T, P>
where
    P: DataProvider<T>,
{
    async fn save(&self) -> Result<(), String> {
        self.provider.update(&self.data).await
    }

    async fn delete(self) -> Result<(), String> {
        self.provider.delete(self.data).await
    }
}
