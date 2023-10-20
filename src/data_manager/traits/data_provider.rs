use std::ops::{Deref, DerefMut};

use uuid::Uuid;

/// Provides basic CRUD operations for a type's data source
///
/// Unless you have a specific use case that warrants a custom implementation,
/// it is recommended that you use one of the premade DataProviders for typical
/// use cases (examples below). A long-term goal is to define the configurability
/// of these to handle contexts / user permissions / etc.
///
///  - [ ] PostgresDataProvider      - Manages the
///  - [ ] TODO: RestApiDataProvider
///  - [ ] TODO: LocalFileDataProvider
///  - [ ] TODO: S3DataProvider
///  - [ ] TODO: InMemoryDataProvider // Intended for debugging/development
///                                   // primarily, but may have other uses (e.g. used in caching)
///  - [ ] TODO: CachedDataProvider<P: DataProvider>
///  - { } TODO: PolicyEnforcedDataProvider
///  - [ ] TODO: MultiSourceDataProvider
pub trait DataProvider<T>
where
    Self: Clone + Send,
{
    type CreateRequest: Default;
    type QueryType: Into<Vec<T>>;

    // fn all(&self) -> Vec<T>;
    fn all(&self) -> Self::QueryType;
    fn get(
        &self,
        id: Uuid,
    ) -> Option<T>;
    fn create(
        &self,
        item: Self::CreateRequest,
    ) -> Result<T, String>; // TODO: Create real error type when needed
    fn delete(
        &self,
        item: T,
    ) -> Result<(), String>;
    fn update(
        &self,
        item: &T,
    ) -> Result<(), String>;
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
    fn save(&self) -> Result<(), String> {
        self.provider.update(&self.data)
    }

    fn delete(self) -> Result<(), String> {
        self.provider.delete(self.data)
    }
}
