use super::table_definition::DatabaseTableDefinition;

#[derive(Clone)]
pub struct DatabaseDefinition {
    pub name: String,
    pub tables: Vec<DatabaseTableDefinition>,
}

impl DatabaseDefinition {
    pub fn new(
        name: String,
        tables: Vec<DatabaseTableDefinition>,
    ) -> Self {
        Self {
            name,
            tables,
        }
    }
}
