mod filters;
mod query_builder;
pub use filters::*;
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
                    parent_table_name: Identifier::new("products".to_string()).unwrap(),
                    column_name: Identifier::new("id".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Uuid,
                    constraints: vec![
                        TableColumnConstraint::primary_key(),
                        TableColumnConstraint::non_null(),
                    ],
                },
                TableColumn {
                    parent_table_name: Identifier::new("products".to_string()).unwrap(),
                    column_name: Identifier::new("description".to_string()).unwrap(),
                    column_type: DatabaseColumnType::String,
                    constraints: vec![
                        TableColumnConstraint::primary_key(),
                        TableColumnConstraint::non_null(),
                    ],
                },
                TableColumn {
                    parent_table_name: Identifier::new("products".to_string()).unwrap(),
                    column_name: Identifier::new("name".to_string()).unwrap(),
                    column_type: DatabaseColumnType::String,
                    constraints: vec![
                        TableColumnConstraint::primary_key(),
                        TableColumnConstraint::non_null(),
                    ],
                },
                TableColumn {
                    parent_table_name: Identifier::new("products".to_string()).unwrap(),
                    column_name: Identifier::new("created_at".to_string()).unwrap(),
                    column_type: DatabaseColumnType::Timestamp,
                    constraints: vec![TableColumnConstraint::non_null()],
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
        DatabaseColumnType, DatabaseTableDefinition, Identifier, TableColumn, TableColumnConstraint,
    };

    #[test]
    fn test() {
        // let query = Product::all().filter(create_date.before(DateTime::now()));
    }
}
