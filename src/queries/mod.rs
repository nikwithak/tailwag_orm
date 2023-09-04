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
        DatabaseTableDefinition::new("products")
            .unwrap()
            .column(TableColumn::uuid("id").unwrap().pk().non_null())
            .column(TableColumn::string("description").unwrap().pk().non_null())
            .column(TableColumn::string("name").unwrap().pk().non_null())
            .column(TableColumn::timestamp("created_at").unwrap().non_null())
            .into()
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
