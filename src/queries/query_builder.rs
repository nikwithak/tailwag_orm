use crate::{database_definition::table_definition::DatabaseTableDefinition, AsSql};

use super::Filter;

pub struct Query<T: Queryable> {
    pub table: DatabaseTableDefinition,
    pub filter: Option<Filter>,
    pub limit: Option<usize>,
    pub _t: Option<T>,
}

trait Saveable<T> {
    fn save(item: T) -> Result<(), String>;
}

trait Deletable<T> {
    fn delete(self) -> Result<(), String>;
}

impl<T: Queryable> AsSql for Query<T> {
    fn as_sql(&self) -> String {
        // TODO: This needs to be unique, add millis or find a better way. What does SQLX do?
        // TODO: This is part of the half-written prepared statement support
        // let query_name = format!("{}_{}", Utc::now().format("%Y%m%d%H%M%S"), self.table_name);

        // TODO: Go through filters, build list of inputs ($1) -> values (literals)
        let mut preproc_stack = Vec::new();
        let mut postproc_stack = Vec::new();
        if let Some(f) = &self.filter {
            preproc_stack.push(f);
        }
        while let Some(f) = preproc_stack.pop() {
            match f {
                Filter::And(children) | Filter::Or(children) => {
                    if children.len() == 0 {
                        // Skip any empty filters - they'll mess up the SQL
                        log::warn!("Empty filter found");
                        continue;
                    }
                    for child in children {
                        preproc_stack.push(child);
                    }
                },
                Filter::Equal(lhs, rhs)
                | Filter::NotEqual(lhs, rhs)
                | Filter::Like(lhs, rhs)
                | Filter::LessThan(lhs, rhs)
                | Filter::LessThanOrEqual(lhs, rhs)
                | Filter::GreaterThan(lhs, rhs)
                | Filter::GreaterThanOrEqual(lhs, rhs) => (),
                // TODO: Figure out what preprocessing needs doing here
                Filter::In(lhs, rhs) => (),
            }
            postproc_stack.push(f);
        }

        // TODO: Add support for joins with filters
        let mut sql = format!("SELECT * FROM {}", &self.table.table_name);

        match &self.filter {
            None => (), // No Where clause if no filters
            Some(filter) => {
                sql.push_str(" WHERE ");
                sql.push_str(&filter.as_sql())
            },
        }

        // TODO: Make prepped statements work
        // let prepared_stmt = format!("PREPARE {} AS {};", query_name, select_query,);
        let prepared_stmt = sql;

        prepared_stmt
    }
}

impl<T: Queryable> Query<T> {
    // fn execute() -> Vec<T> {
    //     todo!()
    // }

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

pub trait Queryable {}
