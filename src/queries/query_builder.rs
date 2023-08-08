use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use chrono::Utc;
use uuid::Uuid;

use crate::{
    database_definition::table_definition::{Identifier, TableColumn},
    AsSql,
};

struct Query<T: Queryable> {
    table_name: Identifier,
    filter: Filter,
    limit: Option<usize>,
    _t: T,
}

impl<T: Queryable> AsSql for Query<T> {
    fn as_sql(&self) -> String {
        // TODO: This needs to be unique, add millis or find a better way. What does SQLX do?
        let query_name = format!("{}_{}", Utc::now().format("%Y%m%d%H%M%S"), self.table_name);

        // TODO: Go through filters, build list of inputs ($1) -> values (literals)
        let mut preproc_stack = Vec::new();
        let mut postproc_stack = Vec::new();
        preproc_stack.push(&self.filter);
        while let Some(f) = preproc_stack.pop() {
            match f {
                Filter::And(children) | Filter::Or(children) => {
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

        // Now grok the stack in reverse to build the query.
        let mut where_clauses = Vec::new();
        while let Some(f) = postproc_stack.pop() {
            match f {
                Filter::And(_) => todo!(),
                Filter::Or(_) => todo!(),
                _ => where_clauses.push(f.as_sql()),
            };
        }

        // TODO: Add support for joins with filters
        let select_query =
            format!("SELECT * FROM {} WHERE {}", &self.table_name, self.filter.as_sql()); // TODO: Remove * in favor of a real type definition

        let prepared_stmt = format!("PREPARE {} AS {};", query_name, select_query,);

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

    fn filter(
        mut self,
        filter: Filter,
    ) -> Self {
        self.filter &= filter;
        self
    }
}

trait Queryable {}

#[derive(Clone)]
enum FilterComparisonParam {
    TableColumn(TableColumn),
    String(String),
    Uuid(Uuid),
    Integer(i64),
    Float(f64),
    Bool(bool),
}

impl AsSql for FilterComparisonParam {
    fn as_sql(&self) -> String {
        match self {
            FilterComparisonParam::TableColumn(col) => {
                // TODO: Need to bring in table_name for longer queries
                format!("{}", &col.column_name)
            },
            FilterComparisonParam::String(s) => format!("\"{}\"", s.to_string()),
            FilterComparisonParam::Uuid(uuid) => format!("\"{}\"", uuid.to_string()),
            FilterComparisonParam::Integer(i) => i.to_string(), // TODO: This'll change when we do prepped statmeents anyway
            FilterComparisonParam::Float(f) => f.to_string(),
            FilterComparisonParam::Bool(b) => b.to_string(),
        }
    }
}

#[derive(Clone)]
enum Filter {
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Equal(FilterComparisonParam, FilterComparisonParam), // All types
    NotEqual(FilterComparisonParam, FilterComparisonParam), // All types
    Like(FilterComparisonParam, FilterComparisonParam),  // Strings only
    LessThan(FilterComparisonParam, FilterComparisonParam), // Non-String types (dates, numbers)
    LessThanOrEqual(FilterComparisonParam, FilterComparisonParam), // Non-String types (dates, numbers)
    GreaterThan(FilterComparisonParam, FilterComparisonParam),     // Non-String types
    GreaterThanOrEqual(FilterComparisonParam, FilterComparisonParam), // Non-String types
    In(FilterComparisonParam, Vec<FilterComparisonParam>),         // All types
}

trait Likeable {
    fn like<T: Into<String>>(
        &self,
        t: T,
    ) -> Filter {
        Filter::Like(
            FilterComparisonParam::TableColumn(todo!()),
            FilterComparisonParam::String(t.into()),
        )
    }
}

impl BitAndAssign for Filter {
    fn bitand_assign(
        &mut self,
        rhs: Self,
    ) {
        *self = self.clone() & rhs;
    }
}

impl BitOrAssign for Filter {
    fn bitor_assign(
        &mut self,
        rhs: Self,
    ) {
        *self = self.clone() | rhs;
    }
}

// TODO: Refactor BitOr and BitAnd to actually merge them together, instead of wrapping them in a higher level construct - applies only if either one is already an Or/And
impl BitOr for Filter {
    type Output = Filter;

    fn bitor(
        self,
        rhs: Self,
    ) -> Self::Output {
        Filter::Or(vec![self, rhs])
    }
}

impl BitAnd for Filter {
    type Output = Filter;

    fn bitand(
        self,
        rhs: Self,
    ) -> Self::Output {
        Filter::And(vec![self, rhs])
    }
}

impl AsSql for Filter {
    fn as_sql(&self) -> String {
        match self {
            Filter::And(children) => {
                // TODO: This is a recursive solution, refactor to iterative one if it causes issues.
                format!(
                    "({})",
                    children
                        .iter()
                        .map(|child| child.as_sql())
                        .collect::<Vec<String>>()
                        .join(" AND "),
                )
            },
            Filter::Or(children) => {
                // TODO: This is a recursive solution, refactor to iterative one if it causes issues.
                format!(
                    "({})",
                    children
                        .iter()
                        .map(|child| child.as_sql())
                        .collect::<Vec<String>>()
                        .join(" OR "),
                )
            },
            Filter::Equal(lhs, rhs) => {
                // TODO: This is UNSAFE for production, open to injection attacks. Need to do pre-processing, might have to rework how filter as_sql works
                format!("({} = {})", lhs.as_sql(), rhs.as_sql(),)
            },
            Filter::NotEqual(lhs, rhs) => {
                format!("({} != {})", lhs.as_sql(), rhs.as_sql())
            },
            Filter::Like(lhs, rhs) => {
                format!("({} LIKE {})", lhs.as_sql(), rhs.as_sql())
            },
            Filter::LessThan(lhs, rhs) => {
                format!("({} < {})", lhs.as_sql(), rhs.as_sql())
            },
            Filter::LessThanOrEqual(lhs, rhs) => {
                format!("({} <= {})", lhs.as_sql(), rhs.as_sql())
            },
            Filter::GreaterThan(lhs, rhs) => {
                format!("({} > {})", lhs.as_sql(), rhs.as_sql())
            },
            Filter::GreaterThanOrEqual(lhs, rhs) => {
                format!("({} >= {})", lhs.as_sql(), rhs.as_sql())
            },
            Filter::In(lhs, rhs) => {
                format!(
                    "({} IN ({}))",
                    lhs.as_sql(),
                    rhs.iter().map(|f| f.as_sql()).collect::<Vec<String>>().join(",")
                )
            },
        }
    }
}
