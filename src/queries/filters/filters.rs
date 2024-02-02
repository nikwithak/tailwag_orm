use crate::{data_definition::table::TableColumn, AsSql};
use sqlx::{query_builder, Postgres, QueryBuilder};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use uuid::Uuid;

#[derive(Clone)]
pub enum FilterComparisonParam {
    TableColumn(TableColumn),
    String(String),
    Uuid(Uuid),
    Integer(i64),
    Float(f64),
    Bool(bool),
}

impl FilterComparisonParam {
    pub fn build_query(
        &self,
        builder: &mut QueryBuilder<Postgres>,
    ) {
        match self {
            FilterComparisonParam::TableColumn(col) => builder.push(&col.column_name),
            FilterComparisonParam::String(val) => builder.push_bind(val.to_string()),
            FilterComparisonParam::Uuid(val) => builder.push_bind(*val),
            FilterComparisonParam::Integer(val) => builder.push_bind(*val),
            FilterComparisonParam::Float(val) => builder.push_bind(*val),
            FilterComparisonParam::Bool(val) => builder.push_bind(*val),
        };
    }
}

impl AsSql for FilterComparisonParam {
    fn as_sql(&self) -> String {
        match self {
            FilterComparisonParam::TableColumn(col) => {
                // TODO: Need to bring in table_name for longer queries
                format!("{}", &col.column_name)
            },
            // Currently this is the only necessary sanitation - rust typechcking sensures we are getting valid types for others.
            // To prevent SQL injection attacks, we replace all single quotes with doubled single-quotes.
            // This should be plenty safe, but in the future consider migrating to prepared statements as an added precaution.
            // Also TODO: TESTS TESTS TESTS
            FilterComparisonParam::String(s) => format!("'{}'", s.replace('\'', "''")), // Escape ' characters. This *should* prevent SQL injection attacks across the board, but needs to be tested and vetted.
            FilterComparisonParam::Uuid(uuid) => format!("'{}'", uuid),
            FilterComparisonParam::Integer(i) => i.to_string(), // TODO: This'll change when we do prepped statmeents anyway
            FilterComparisonParam::Float(f) => f.to_string(),
            FilterComparisonParam::Bool(b) => b.to_string(),
        }
    }
}

#[derive(Clone)]
// TODO: Make Filters associated with their tables
// There's a lot more to do with the filters here - nailing this down is gonna be super powerful
// Will work with PG stuff, and also let me piecemeal replace the functionality...
pub enum Filter {
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

impl Filter {
    fn get_operator(&self) -> &str {
        match self {
            Filter::And(_) => "AND",
            Filter::Or(_) => "OR",
            Filter::Equal(_, _) => "=",
            Filter::NotEqual(_, _) => "!=",
            Filter::Like(_, _) => "LIKE",
            Filter::LessThan(_, _) => "<",
            Filter::LessThanOrEqual(_, _) => "<=",
            Filter::GreaterThan(_, _) => ">",
            Filter::GreaterThanOrEqual(_, _) => ">=",
            Filter::In(_, _) => "IN",
        }
    }
}

// trait Likeable {
//     fn like<T: Into<String>>(
//         &self,
//         _t: T,
//     ) -> Filter {
//         Filter::Like(
//             FilterComparisonParam::TableColumn(todo!()),
//             FilterComparisonParam::String(_t.into()),
//         )
//     }
// }

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
        match self {
            Filter::Or(mut existing_ors) => {
                existing_ors.push(rhs);
                Filter::Or(existing_ors)
            },
            _ => Filter::Or(vec![self, rhs]),
        }
    }
}

impl BitAnd for Filter {
    type Output = Filter;
    fn bitand(
        self,
        rhs: Self,
    ) -> Self::Output {
        match self {
            Filter::And(mut existing_ors) => {
                existing_ors.push(rhs);
                Filter::And(existing_ors)
            },
            _ => Filter::And(vec![self, rhs]),
        }
    }
}

impl Filter {
    pub fn build_query(
        &self,
        builder: &mut QueryBuilder<Postgres>,
    ) {
        match self {
            Filter::And(children) | Filter::Or(children) => {
                let mut iter = children.iter().peekable();
                while let Some(child) = iter.next() {
                    child.build_query(builder);
                    if iter.peek().is_some() {
                        builder.push(" AND ");
                    }
                }
            },
            Filter::Equal(l, r)
            | Filter::NotEqual(l, r)
            | Filter::Like(l, r)
            | Filter::LessThan(l, r)
            | Filter::LessThanOrEqual(l, r)
            | Filter::GreaterThan(l, r)
            | Filter::GreaterThanOrEqual(l, r) => {
                l.build_query(builder);
                builder.push(" ");
                builder.push(self.get_operator());
                builder.push(" ");
                r.build_query(builder);
            },
            Filter::In(_val, _list) => {
                todo!()
            },
        }
    }
}

impl AsSql for Filter {
    fn as_sql(&self) -> String {
        type F = Filter;
        match self {
            F::And(children) => {
                if children.is_empty() {
                    return "true".to_string();
                }
                format!(
                    "({})",
                    children
                        .iter()
                        .map(|child| child.as_sql())
                        .collect::<Vec<String>>()
                        .join(" AND "),
                )
            },
            F::Or(children) => {
                if children.is_empty() {
                    return "".to_string();
                }
                format!(
                    "({})",
                    children
                        .iter()
                        .map(|child| child.as_sql())
                        .collect::<Vec<String>>()
                        .join(" OR "),
                )
            },
            F::Equal(lhs, rhs) => {
                // TODO: This is UNSAFE for production, open to injection attacks. Need to do pre-processing, might have to rework how filter as_sql works
                format!("({} = {})", lhs.as_sql(), rhs.as_sql(),)
            },
            F::NotEqual(lhs, rhs) => {
                format!("({} != {})", lhs.as_sql(), rhs.as_sql())
            },
            F::Like(lhs, rhs) => {
                format!("({} LIKE {})", lhs.as_sql(), rhs.as_sql())
            },
            F::LessThan(lhs, rhs) => {
                format!("({} < {})", lhs.as_sql(), rhs.as_sql())
            },
            F::LessThanOrEqual(lhs, rhs) => {
                format!("({} <= {})", lhs.as_sql(), rhs.as_sql())
            },
            F::GreaterThan(lhs, rhs) => {
                format!("({} > {})", lhs.as_sql(), rhs.as_sql())
            },
            F::GreaterThanOrEqual(lhs, rhs) => {
                format!("({} >= {})", lhs.as_sql(), rhs.as_sql())
            },
            F::In(lhs, rhs) => {
                format!(
                    "({} IN ({}))",
                    lhs.as_sql(),
                    rhs.iter().map(|f| f.as_sql()).collect::<Vec<String>>().join(",")
                )
            },
        }
    }
}
