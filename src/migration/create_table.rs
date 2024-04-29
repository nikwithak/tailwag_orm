use sqlx::Postgres;

use crate::{data_definition::table::DatabaseTableDefinition, AsSql, BuildSql};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CreateTable<T> {
    table_definition: DatabaseTableDefinition<T>,
}

impl<T> CreateTable<T> {
    pub fn new(table: DatabaseTableDefinition<T>) -> Self {
        Self {
            table_definition: table,
        }
    }

    pub fn table_definition(&self) -> &DatabaseTableDefinition<T> {
        &self.table_definition
    }
}

impl<T> BuildSql for CreateTable<T> {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        sql.push("CREATE TABLE IF NOT EXISTS ");
        // sql.push_bind(self.table_definition.table_name.to_string());
        sql.push(self.table_definition.table_name.to_string());
        sql.push(" (");
        let mut columns = self.table_definition.columns.values().peekable();
        while let Some(column) = columns.next() {
            column.build_sql(sql);
            if columns.peek().is_some() {
                sql.push(",");
            }
        }
        sql.push(");");
    }
}

impl<T> AsSql for CreateTable<T> {
    fn as_sql(&self) -> String {
        let mut builder = sqlx::QueryBuilder::new("");
        self.build_sql(&mut builder);
        builder.into_sql()
    }
}

#[cfg(test)]
mod test {
    use create_table::CreateTable;

    use crate::{
        data_definition::table::{DatabaseTableDefinition, Identifier, TableColumn},
        migration::create_table,
        AsSql,
    };

    #[test]
    fn as_sql_foreign_key_on_column_works() {
        todo!()
    }

    #[test]
    fn as_sql_works() -> Result<(), String> {
        let table_name = Identifier::new("new_table".to_string()).unwrap();
        let table_definition = DatabaseTableDefinition::<()>::new(&table_name)
            .unwrap()
            .column(TableColumn::new_uuid("uuid_pk_nonnull")?.non_null().pk())
            .column(TableColumn::new_string("string")?)
            .column(TableColumn::new_bool("bool_nonnull")?.non_null())
            .column(TableColumn::new_float("float_nonnull")?.non_null())
            .column(TableColumn::new_int("int")?)
            .column(TableColumn::new_timestamp("create_timestamp")?)
            .into();

        let create_table = CreateTable {
            table_definition,
        };

        // Act
        let queries = dbg!(create_table.as_sql());
        // let mut queries = result_sql.split("\n").collect::<Vec<&str>>();

        #[rustfmt::skip]
        let expected_query = ["CREATE TABLE IF NOT EXISTS new_table (",
                "bool_nonnull BOOL NOT NULL,",
                "create_timestamp TIMESTAMP,",
                "float_nonnull FLOAT NOT NULL,",
                "int INT,",
                "string VARCHAR,",
                "uuid_pk_nonnull UUID NOT NULL PRIMARY KEY",
            " );"].join("");

        assert_eq!(queries, expected_query);
        Ok(())
    }
}
