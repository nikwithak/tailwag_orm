mod query_builder;
pub use query_builder::*;

#[cfg(test)]
mod tests {

    struct Product {
        id: Uuid,
        name: String,
        description: String,
        create_time: chrono::DateTime<Utc>,
    }

    enum FilterableType {
        Uuid,
        String,
        Boolean,
        DateTime,
    }

    struct ProductFilters {
        id: Uuid,
        name: String,
        description: String,
        create_time: chrono::DateTime<Utc>,
    }

    impl ProductFilters {}

    fn get_product_table_def() -> DatabaseTableDefinition {
        DatabaseTableDefinition {
            table_name: Identifier::new("products".to_string()).unwrap(),
            columns: vec![
                TableColumn {
                    // TODO: Is it worth it to make this Identifier a ref back to parent?
                    parent_table_name: Identifier::new("products".to_string()).unwrap(),
                    column_name: Identifier::new("id".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Uuid,
                    is_primary_key: true,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("products".to_string()).unwrap(),
                    column_name: Identifier::new("description".to_string()).unwrap(),
                    column_type: DatabaseColumnType::String,
                    is_primary_key: true,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("products".to_string()).unwrap(),
                    column_name: Identifier::new("name".to_string()).unwrap(),
                    column_type: DatabaseColumnType::String,
                    is_primary_key: true,
                    is_nullable: false,
                },
                TableColumn {
                    parent_table_name: Identifier::new("products".to_string()).unwrap(),
                    column_name: Identifier::new("created_at".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Timestamp,
                    is_primary_key: false,
                    is_nullable: false,
                },
            ],
        }
    }
    // impl Queryable for Product {}

    // impl Product {
    //     pub fn all() -> QueryBuilder<Product> {
    //         todo!()
    //     }
    // }

    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    use crate::database_definition::table_definition::{
        DatabaseColumnType, DatabaseTableDefinition, Identifier, TableColumn,
    };

    #[test]
    fn test() {
        // let query = Product::all().filter(create_date.before(DateTime::now()));
    }
}
