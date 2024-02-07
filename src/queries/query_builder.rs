use std::marker::PhantomData;

use crate::{
    data_definition::table::DatabaseTableDefinition,
    object_management::{
        delete::DeleteStatement, insert::InsertStatement, update::UpdateStatement,
    },
    AsSql,
};

use super::Filter;

pub struct Query<T> {
    pub table: DatabaseTableDefinition<T>,
    pub filter: Option<Filter>,
    pub limit: Option<usize>,
    pub _t: PhantomData<T>,
}

pub trait Saveable {
    // TODO: customize error type
    fn save(&self) -> Result<(), String>;
}

pub trait Deleteable {
    // TODO: rework this to actually Delete, not get_delete_statement
    fn get_delete_statement(&self) -> DeleteStatement<Self>
    where
        Self: std::marker::Sized;
}

pub trait Updateable {
    fn get_update_statement(&self) -> UpdateStatement<Self>
    where
        Self: std::marker::Sized;
}

impl<T> Query<T> {
    // fn execute() -> Vec<T> {
    //     todo!()
    // }

    #[allow(unused)]
    fn limit(
        mut self,
        limit: usize,
    ) -> Self {
        self.limit = Some(limit);
        self
    }

    // fn get(filter: Filter) -> Option<T> {
    //     // TODO: get by ID
    //     todo!()
    // }
    pub fn filter(
        mut self,
        filter: Filter,
    ) -> Self {
        if let Some(f) = self.filter {
            self.filter = Some(f & filter)
        } else {
            self.filter = Some(filter)
        }
        self
    }
}

/// DEPRECATED - left in for backwards compatibility.
/// TODO: Remove when old types are updated.
// pub trait Qeryable {}

pub trait Insertable
where
    Self: Sized,
{
    fn get_insert_statement(&self) -> InsertStatement<Self>;
}

#[cfg(test)]
mod tests {
    use crate::data_definition::table::{DatabaseTableDefinition, TableColumn};

    fn get_table_def<T>() -> DatabaseTableDefinition<T> {
        type T = TableColumn;

        DatabaseTableDefinition::new("table_2")
            .unwrap()
            .column(T::string("string_nullable").unwrap())
            .column(T::bool("bool").unwrap().non_null())
            .column(T::int("int").unwrap().non_null())
            .column(T::float("float").unwrap().non_null())
            .column(T::timestamp("timestamp").unwrap().non_null())
            .column(T::uuid("id").unwrap().non_null())
            .into()
    }
    #[test]
    fn test_queries() {
        const _EXPECTED_QUERY: &str = "SELECT * FROM item INNER JOIN sub_item ON sub_item.parent_id = item.id WHERE sub_item.name like 'BUG%';";

        let _table_def: DatabaseTableDefinition<()> = get_table_def();
        // let _query = Query::<String

        todo!()
    }
}
