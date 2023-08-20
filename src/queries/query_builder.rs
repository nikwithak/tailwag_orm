use crate::{database_definition::table_definition::Identifier, AsSql};

use super::Filter;

pub struct Query<T: Queryable> {
    pub table_name: Identifier,
    pub filter: Filter,
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
        preproc_stack.push(&self.filter);
        while let Some(f) = preproc_stack.pop() {
            match f {
                Filter::And(children) | Filter::Or(children) => {
                    if children.len() == 0 {
                        // Skip any empty filters - they'll mess up the SQL
                        continue;
                    }
                    for child in children {
                        preproc_stack.push(child);
                    }
                },
                _ => {}, // Ignore any comparisons - we'll catch it on t)he next pass, when we go backwards.
            }
            postproc_stack.push(f);
        }
        // Would this be easier if I just did it recursively instead of trying to build an iterative solution?
        // Depends if call stack will ever get deep enough (with a huuuuuuuuge amount of nested ands/ors)

        // // Now grok the stack in reverse to build the query.
        // let mut where_clauses = Vec::new();
        // while let Some(f) = postproc_stack.pop() {
        //     match f {
        //         Filter::And(_) => todo!(),
        //         Filter::Or(_) => todo!(),
        //         _ => where_clauses.push(f.as_sql()),
        //     };
        // }

        // TODO: Add support for joins with filters
        let select_query =
            format!("SELECT * FROM {} WHERE {}", &self.table_name, self.filter.as_sql());
        // TODO: Remove * in favor of a real type definition
        // format!("SELECT * FROM {};", &self.table_name);
        // TODO: Remove * in favor of a real type definition

        // TODO: Make prepped statements work
        // let prepared_stmt = format!("PREPARE {} AS {};", query_name, select_query,);
        let prepared_stmt = select_query;

        prepared_stmt
    }
}

impl<T: Queryable> Query<T> {
    fn execute() -> Vec<T> {
        todo!()
    }

    fn limit(
        mut self,
        limit: usize,
    ) -> Self {
        self.limit = Some(limit);
        self
    }

    fn get(filter: Filter) -> Option<T> {
        // TODO: get by ID
        todo!()
    }

    pub fn filter(
        mut self,
        filter: Filter,
    ) -> Self {
        self.filter &= filter;
        self
    }
}

pub trait Queryable {}
