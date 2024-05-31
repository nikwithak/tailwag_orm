use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::marker::PhantomData;

use crate::{
    data_definition::table::{DatabaseTableDefinition, Identifier},
    object_management::{
        delete::DeleteStatement, insert::InsertStatement, update::UpdateStatement,
    },
    BuildSql,
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
    fn get_update_statement(&self) -> Vec<UpdateStatement>
    where
        Self: std::marker::Sized;
}

impl<T> Query<T> {
    #[allow(unused)]
    fn limit(
        mut self,
        limit: usize,
    ) -> Self {
        self.limit = Some(limit);
        self
    }

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

pub trait Insertable
where
    Self: Sized,
{
    type CreateRequest: Default + Serialize + for<'a> Deserialize<'a> + Into<Self>;

    fn get_insert_statement(&self) -> Vec<InsertStatement>;
}

impl<T> BuildSql for Query<T> {
    fn build_sql(
        &self,
        query_builder: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        let table_name = self.table.table_name.clone();
        let mut group_by: Vec<String> = vec![format!("{}.id", &self.table.table_name)];
        type E = crate::data_definition::table::DatabaseColumnType;
        // STEP ONE: get all table relationships
        let mut attrs = self
            .table
            .columns
            .values()
            .map(|col| {
                let col_name = col.column_name.to_string();
                match col.column_type {
                    E::Boolean
                    | E::Int
                    | E::Float
                    | E::String
                    | E::Timestamp
                    | E::Uuid
                    | E::Json => format!("{table_name}.{col_name}"),
                    E::OneToMany(_) => todo!(),
                    E::ManyToMany(_) => todo!(),
                    E::OneToOne(_) => {
                        col_name.trim_end_matches("_id").to_string() // TODO: UNHACK THIS
                    },
                }
            })
            .peekable();
        query_builder.push(r"SELECT ");
        while let Some(attr) = attrs.next() {
            query_builder.push(attr);
            if attrs.peek().is_some() {
                query_builder.push(", ");
            }
        }
        query_builder.push(" FROM ");
        query_builder.push(&table_name);
        // TODO: Inner Joins -
        // STEP THREE: Need to impl BuildSql for INNER JOIN
        for child_tbl in self.table.columns.values() {
            match &child_tbl.column_type {
                crate::data_definition::table::DatabaseColumnType::OneToMany(name)
                | crate::data_definition::table::DatabaseColumnType::ManyToMany(name)
                | crate::data_definition::table::DatabaseColumnType::OneToOne(name) => {
                    let name = name.strip_suffix("_id").unwrap(); // TODO: UNHACK THIS
                    group_by.push(name.to_string());
                    query_builder
                        .push(" LEFT OUTER JOIN ")
                        .push(name)
                        .push(" ON ")
                        .push(name)
                        .push(".id = ")
                        .push(name)
                        .push("_id ");
                },
                _ => {},
            };
        }
        if let Some(filter) = &self.filter {
            query_builder.push(" WHERE ");
            filter.build_sql(query_builder);
        }

        // TODO: Unhack (part of the "everything built on id" problem)
        query_builder.push(" GROUP BY (").push(group_by.join(", ")).push(")");

        // STEP FOUR: Probably will need to do more with build_query_as. will find out
    }
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
