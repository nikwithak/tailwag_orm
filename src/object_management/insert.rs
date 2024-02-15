use std::collections::HashMap;

use sqlx::Postgres;

use crate::{
    data_definition::table::{ColumnValue, DatabaseTableDefinition, Identifier},
    AsSql, BuildSql,
};

pub struct InsertStatement<T> {
    table_def: DatabaseTableDefinition<T>,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    object_repr: HashMap<Identifier, ColumnValue>,
}

impl<T> InsertStatement<T> {
    pub fn new(
        table_def: DatabaseTableDefinition<T>,
        object_map: HashMap<Identifier, ColumnValue>,
    ) -> Self {
        Self {
            table_def,
            object_repr: object_map,
        }
    }
}

impl<T> BuildSql for InsertStatement<T> {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        let mut columns: Vec<&str> = Vec::new();
        let mut values = Vec::new();
        for (column, value) in &self.object_repr {
            columns.push(column);
            values.push(value);
        }
        builder.push(format!(
            "INSERT INTO {} ({}) VALUES (",
            self.table_def.table_name,
            &columns.join(", "),
        ));
        for (i, value) in values.iter().enumerate() {
            match value {
                ColumnValue::Boolean(val) => builder.push_bind(*val),
                ColumnValue::Int(val) => builder.push_bind(*val),
                ColumnValue::Float(val) => builder.push_bind(*val),
                ColumnValue::String(val) | ColumnValue::Json(val) => {
                    builder.push_bind(val.to_string())
                },
                ColumnValue::Timestamp(val) => builder.push_bind(*val),
                ColumnValue::Uuid(val) => builder.push_bind(*val),
            };
            if i < values.len() - 1 {
                builder.push(", ");
            }
        }
        builder.push(");");
    }
}
