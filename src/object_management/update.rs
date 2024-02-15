use std::collections::HashMap;

use sqlx::Postgres;

use crate::{AsSql, BuildSql};

use crate::data_definition::table::{ColumnValue, DatabaseTableDefinition, Identifier};

pub struct UpdateStatement<T> {
    table_def: DatabaseTableDefinition<T>,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    object_repr: HashMap<Identifier, ColumnValue>,
}

impl<T> UpdateStatement<T> {
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

impl<T> BuildSql for UpdateStatement<T> {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        builder.push(format!("UPDATE {} SET ", self.table_def.table_name,));

        let Some(ColumnValue::Uuid(id)) = self.object_repr.get(&Identifier::new_unchecked("id"))
        else {
            panic!("Received update without ID - shouldn't be possble.")
        };
        for (i, (column, value)) in self
            .object_repr
            .iter()
            .filter(|(k, v)| &k as &str != &Identifier::new_unchecked("id") as &str)
            .enumerate()
        {
            match value {
                ColumnValue::Boolean(val) => builder.push(column).push(" = ").push_bind(*val),
                ColumnValue::Int(val) => builder.push(column).push(" = ").push_bind(*val),
                ColumnValue::Float(val) => builder.push(column).push(" = ").push_bind(*val),
                ColumnValue::String(val) | ColumnValue::Json(val) => {
                    builder.push(column).push(" = ").push_bind(val.to_string())
                },
                ColumnValue::Timestamp(val) => builder.push(column).push(" = ").push_bind(*val),
                ColumnValue::Uuid(val) => builder.push(column).push(" = ").push_bind(*val),
            };
            if i < &self.object_repr.len() - 2 {
                // -2 because we've stripped `id` from it above.
                builder.push(", ");
            }
        }
        builder.push(" WHERE id=").push_bind(*id);
    }
}
