use sqlx::Postgres;

use crate::{
    data_definition::table::{ColumnValue, Identifier, ObjectRepr},
    BuildSql,
};

pub struct InsertStatement {
    table_name: Identifier,
    // TODO: Make this a little more specific? Good enough for now (probably), but needs to be thoroughly tested
    object_repr: ObjectRepr,
}

impl InsertStatement {
    pub fn new(
        table_name: Identifier,
        object_map: ObjectRepr,
    ) -> Self {
        Self {
            table_name,
            object_repr: object_map,
        }
    }
}

// WITH 

impl BuildSql for InsertStatement {
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
        builder
            .push(format!("INSERT INTO {} ({}) VALUES (", self.table_name, &columns.join(", "),));
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
                ColumnValue::Child(values) => todo!("It's getting messy."),
            };
            if i < values.len() - 1 {
                builder.push(", ");
            }
        }
        builder.push(");");
    }
}
