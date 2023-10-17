use std::ops::{Deref, DerefMut};

use uuid::Uuid;

pub trait DataProvider<T> {
    type CreateRequest: Into<T>;
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
