use uuid::Uuid;

use crate::queries::Queryable;

use super::ExecutableQuery;

pub trait QueryItems<T: Queryable> {
    fn query(&self) -> ExecutableQuery<T>;
}

pub trait GetItemById<T> {
    fn get_by_id(
        &self,
        id: &Uuid,
    ) -> Result<T, String>;
}

pub trait CreateItem<T> {
    fn create(
        &self,
        item: T,
    ) -> Result<T, String>;
}
