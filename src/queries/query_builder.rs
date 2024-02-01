use std::marker::PhantomData;

use crate::{
    data_definition::table::DatabaseTableDefinition,
    object_management::{
        delete::DeleteStatement, insert::InsertStatement, update::UpdateStatement,
    },
    AsSql,
};

use super::Filter;

pub struct Query<T> {
    pub table: DatabaseTableDefinition,
    pub filter: Option<Filter>,
    pub limit: Option<usize>,
    pub _t: PhantomData<T>,
}

pub trait Saveable {
    // TODO: customize error type
    fn save(&self) -> Result<(), String>;
}

pub trait Deleteable {
    // TODO: rework this to actually Delete, not get_delete_statement
    fn get_delete_statement(&self) -> DeleteStatement;
}

pub trait Updateable {
    fn get_update_statement(&self) -> UpdateStatement;
}

impl<T> AsSql for Query<T> {
    fn as_sql(&self) -> String {
        // TODO: This needs to be unique, add millis or find a better way. What does SQLX do?
        // TODO: This is part of the half-written prepared statement support
        // let query_name = format!("{}_{}", Utc::now().format("%Y%m%d%H%M%S"), self.table_name);

        // TODO: Go through filters, build list of inputs ($1) -> values (literals)
        let mut preproc_stack = Vec::new();
        let mut parameters = Vec::new();
        if let Some(f) = &self.filter {
            preproc_stack.push(f);
        }
        while let Some(f) = preproc_stack.pop() {
            match f {
                Filter::And(children) | Filter::Or(children) => {
                    if children.is_empty() {
                        // Skip any empty filters - they'll mess up the SQL
                        log::warn!("Empty filter found - this could be a bug.");
                        continue;
                    }
                    for child in children {
                        preproc_stack.push(child);
                    }
                },
                // Filter::Equal(_lhs, _rhs)
                // | Filter::NotEqual(_lhs, _rhs)
                // | Filter::Like(_lhs, _rhs)
                // | Filter::LessThan(_lhs, _rhs)
                // | Filter::LessThanOrEqual(_lhs, _rhs)
                // | Filter::GreaterThan(_lhs, _rhs)
                // | Filter::GreaterThanOrEqual(_lhs, _rhs) => todo!(),
                // // TODO: Think about what preprocessing needs doing here
                // Filter::In(_lhs, _rhs) => todo!(),
                _ => {
                    log::info!("Unknown filter skipped: {}", &f.as_sql())
                },
            }
            parameters.push(f);
        }

        // TODO: Add support for joins with filters
        let mut sql = format!("SELECT * FROM {}", &self.table.table_name);

        // WIP (CURRENT): Implementing  the JOIN with filters
        // {
        //     // TODO: Get relationships & relationship types
        //     // One-to-One (Owned) - results in a "parent_id" column on the child table
        //     sql.push_str(
        //         " INNER JOIN {child_table_name} ON {child_table_pk} == {my_item.child_id} ",
        //     );
        //     // OR, for One-to-Many (Also Owned, but in a Vec<>):
        //     // TODO: Need ARRAY_AGG
        //     // TODO: Also add a parent_type if crossing tables? (Future Feature)
        //     sql.push_str(" INNER JOIN {child_table_name} ON {child_table_pk} == {my_table.id} ");
        //     // Need to also make sure that in SELECT we use (ARRAY_AGG) or (JSON_AGG)
        //     // Many-to-Many (Unowned / Shared) - results in a join_table column on the child table
        //     sql.push_str(" INNER JOIN {join_table} ON {ref_id} == {table_ref_id} INNER JOIN {child_table} ON {join_table} ");
        //     // TODO / Consider: Structured / custom join tables (for edge data / metadata?) via a Macro or Wrapper type

        //     // TODO: Make sure that filters (below) accurately reference the oin_table names
        // }

        match &self.filter {
            None => (), // No Where clause if no filters
            Some(filter) => {
                sql.push_str(" WHERE ");
                sql.push_str(&filter.as_sql())
            },
        }

        // TODO: Make prepped statements work
        // let prepared_stmt = format!("PREPARE {} AS {};", query_name, select_query,);

        sql
    }
}

impl<T> Query<T> {
    // fn execute() -> Vec<T> {
    //     todo!()
    // }

    #[allow(unused)]
    fn limit(
        mut self,
        limit: usize,
    ) -> Self {
        self.limit = Some(limit);
        self
    }

    // fn get(filter: Filter) -> Option<T> {
    //     // TODO: get by ID
    //     todo!()
    // }
    pub fn filter(
        mut self,
        filter: Filter,
    ) -> Self {
        if let Some(f) = self.filter {
            self.filter = Some(f & filter)
        } else {
            self.filter = Some(filter)
        }
        self
    }
}

/// DEPRECATED - left in for backwards compatibility.
/// TODO: Remove when old types are updated.
// pub trait Queryable {}

pub trait Insertable {
    fn get_insert_statement(&self) -> InsertStatement;
}

#[cfg(test)]
mod tests {
    use crate::data_definition::table::{DatabaseTableDefinition, TableColumn};

    fn get_table_def() -> DatabaseTableDefinition {
        type T = TableColumn;

        DatabaseTableDefinition::new("table_2")
            .unwrap()
            .column(T::string("string_nullable").unwrap())
            .column(T::bool("bool").unwrap().non_null())
            .column(T::int("int").unwrap().non_null())
            .column(T::float("float").unwrap().non_null())
            .column(T::timestamp("timestamp").unwrap().non_null())
            .column(T::uuid("id").unwrap().non_null())
            .into()
    }
    #[test]
    fn test_queries() {
        const _EXPECTED_QUERY: &str = "SELECT * FROM item INNER JOIN sub_item ON sub_item.parent_id = item.id WHERE sub_item.name like 'BUG%';";

        let _table_def = get_table_def();
        // let _query = Query::<String

        todo!()
    }
}
