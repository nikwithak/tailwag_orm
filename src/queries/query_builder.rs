use std::marker::PhantomData;

use crate::{
    data_definition::table::DatabaseTableDefinition,
    object_management::{
        delete::DeleteStatement, insert::InsertStatement, update::UpdateStatement,
    },
    AsSql,
};

use super::Filter;

pub struct Query<T: Queryable> {
    pub table: DatabaseTableDefinition,
    pub filter: Option<Filter>,
    pub limit: Option<usize>,
    pub _t: PhantomData<T>,
}

impl<T: Queryable> Into<Vec<T>> for Query<T> {
    fn into(self) -> Vec<T> {
        todo!()
    }
}

pub trait Saveable {
    fn save(&self) -> Result<(), String>;
}

pub trait Deleteable {
    fn get_delete_statement(&self) -> DeleteStatement;
}

pub trait Updateable {
    fn get_update_statement(&self) -> UpdateStatement;
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
                Filter::Equal(_lhs, _rhs)
                | Filter::NotEqual(_lhs, _rhs)
                | Filter::Like(_lhs, _rhs)
                | Filter::LessThan(_lhs, _rhs)
                | Filter::LessThanOrEqual(_lhs, _rhs)
                | Filter::GreaterThan(_lhs, _rhs)
                | Filter::GreaterThanOrEqual(_lhs, _rhs) => (),
                // TODO: Think about what preprocessing needs doing here
                Filter::In(_lhs, _rhs) => (),
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

pub trait Queryable {}

pub trait Insertable {
    fn get_insert_statement(&self) -> InsertStatement;
}
