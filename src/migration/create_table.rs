use crate::{data_definition::table::DatabaseTableDefinition, AsSql};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CreateTable {
    table_definition: DatabaseTableDefinition,
}

impl CreateTable {
    pub fn new(table: DatabaseTableDefinition) -> Self {
        Self {
            table_definition: table,
        }
    }

    pub fn table_definition(&self) -> &DatabaseTableDefinition {
        &self.table_definition
    }
}

impl AsSql for CreateTable {
    fn as_sql(&self) -> String {
        let columns_sql = self
            .table_definition
            .columns
            .values()
            .map(|col| col.as_sql())
            .collect::<Vec<String>>()
            .join(",\n");

        // TODO: I'm gonna need to do this in multiple passes. First pass: Create the tables / columns, and second pass: Add the table constraints

        let mut sql =
            format!("CREATE TABLE IF NOT EXISTS {} (\n", self.table_definition.table_name);
        sql.push_str(&columns_sql);
        sql.push_str("\n);");

        sql
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
        let table_definition = DatabaseTableDefinition::new(&table_name)
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
        let queries = create_table.as_sql();
        // let mut queries = result_sql.split("\n").collect::<Vec<&str>>();

        #[rustfmt::skip]
        let expected_query = ["CREATE TABLE IF NOT EXISTS new_table (",
                "uuid_pk_nonnull UUID NOT NULL PRIMARY KEY,",
                "string VARCHAR,",
                "bool_nonnull BOOL NOT NULL,",
                "float_nonnull FLOAT NOT NULL,",
                "int INT,",
                "create_timestamp TIMESTAMP",
            ");"].join("\n");

        assert_eq!(queries, expected_query);
        Ok(())
    }
}
