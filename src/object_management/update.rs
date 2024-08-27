use std::collections::HashMap;

use sqlx::Postgres;

use crate::BuildSql;

use crate::data_definition::table::{ColumnValue, DatabaseTableDefinition, Identifier};

pub struct UpdateStatement {
    table_name: Identifier,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    object_repr: HashMap<Identifier, ColumnValue>,
}

impl UpdateStatement {
    pub fn new(
        table_def: DatabaseTableDefinition,
        object_map: HashMap<Identifier, ColumnValue>,
    ) -> Self {
        Self {
            table_name: table_def.table_name.clone(),
            object_repr: object_map,
        }
    }
}

impl BuildSql for UpdateStatement {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        builder.push(format!("UPDATE {} SET ", self.table_name));

        let Some(ColumnValue::Uuid(id)) = self.object_repr.get(&Identifier::new_unchecked("id"))
        else {
            panic!("Received update without ID - shouldn't be possble.")
        };
        for (i, (column, value)) in self
            .object_repr
            .iter()
            .filter(|(k, _v)| &k as &str != &Identifier::new_unchecked("id") as &str)
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
                ColumnValue::Child(_) => todo!(),
            };
            if i < &self.object_repr.len() - 2 {
                // .len()-2 because we've stripped `id` from it above.
                builder.push(", ");
            }
        }
        builder.push(" WHERE id=").push_bind(dbg!(*id));
    }
}
