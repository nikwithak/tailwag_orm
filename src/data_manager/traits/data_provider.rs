use uuid::Uuid;

use crate::queries::{Insertable, Queryable};

pub trait DataProvider<T> {
    type CreateRequest;

    fn get(id: Uuid) -> Option<T>;
    fn create(item: Self::CreateRequest) -> Result<(), String>; // TODO: Create real error type when needed
    fn delete(item: T) -> Result<(), String>;
    fn all() -> Vec<T>;
    fn update(item: T) -> Result<(), String>;
}
