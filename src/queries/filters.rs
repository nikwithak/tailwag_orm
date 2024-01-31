use crate::{data_definition::table::TableColumn, AsSql};
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

impl AsSql for FilterComparisonParam {
    fn as_sql(&self) -> String {
        match self {
            FilterComparisonParam::TableColumn(col) => {
                // TODO: Need to bring in table_name for longer queries
                format!("{}", &col.column_name)
            },
            FilterComparisonParam::String(s) => format!("'{}'", s),
            FilterComparisonParam::Uuid(uuid) => format!("'{}'", uuid),
            FilterComparisonParam::Integer(i) => i.to_string(), // TODO: This'll change when we do prepped statmeents anyway
            FilterComparisonParam::Float(f) => f.to_string(),
            FilterComparisonParam::Bool(b) => b.to_string(),
        }
    }
}

pub mod filterable_types {
    use std::marker::PhantomData;

    use crate::data_definition::table::{Identifier, TableColumn};

    use super::Filter;

    // Trying out the type-state pattern here.
    trait TypeFilter {}
    macro_rules! typetype {
        ($item:ident) => {
            pub enum $item {}
            impl TypeFilter for $item {}
        };
    }
    macro_rules! impl_filter_for {
        ($item:ident: $base_type:ty, $table_column_fn_name:ident, $param_type_enum:ident, $trait_name:ident $($func_name:ident:$comparison_type:ident),*) => {
            impl $trait_name for FilterableType<$item> {
                type Type = $base_type;
                $(fn $func_name(
                    &self,
                    value: impl Into<<Self as $trait_name>::Type>,
                ) -> Filter {
                    Filter::$comparison_type(
                        super::FilterComparisonParam::TableColumn(
                            TableColumn::$table_column_fn_name(&self.column_name).unwrap().into(),
                        ),
                        super::FilterComparisonParam::$param_type_enum(value.into()),
                    )
                }
                )*
            }
        };
    }

    // Here we define the filterabilities of each type
    typetype! {UuidFilter}
    impl_filter_for!(UuidFilter: uuid::Uuid, new_uuid, Uuid, FilterEq eq:Equal, ne:NotEqual);
    impl_filter_for!(UuidFilter: uuid::Uuid, new_uuid, Uuid, FilterLike like:Like);
    typetype! {BooleanFilter}
    impl_filter_for!(BooleanFilter: bool, new_bool, Bool, FilterEq eq:Equal, ne:NotEqual);
    typetype! {IntFilter}
    impl_filter_for!(IntFilter: i64, new_int, Integer, FilterEq eq:Equal, ne:NotEqual);
    impl_filter_for!(IntFilter: i64, new_int, Integer, FilterPartialEq lt:LessThan, lte:LessThanOrEqual, gt:GreaterThan, gte:GreaterThanOrEqual);
    typetype! {FloatFilter}
    impl_filter_for!(FloatFilter: f64, new_int, Float, FilterEq eq:Equal, ne:NotEqual);
    impl_filter_for!(FloatFilter: f64, new_int, Float, FilterPartialEq lt:LessThan, lte:LessThanOrEqual, gt:GreaterThan, gte:GreaterThanOrEqual);
    typetype! {StringFilter}
    impl_filter_for!(StringFilter: String, new_int, String, FilterEq eq:Equal, ne:NotEqual);
    impl_filter_for!(StringFilter: String, new_int, String, FilterPartialEq lt:LessThan, lte:LessThanOrEqual, gt:GreaterThan, gte:GreaterThanOrEqual);
    impl_filter_for!(StringFilter: String, new_int, String, FilterLike like:Like);
    // impl_filter_for!(TimestampFilter: chrono::NaiveDateTime, new_timestamp, Timestamp, FilterEq eq:Equal, ne:NotEqual); // TODO: Timestamp not supported yet

    pub struct FilterableType<T: TypeFilter> {
        // _table_name: Identifier,
        column_name: Identifier,
        _t: PhantomData<T>,
    }

    impl<T: TypeFilter> FilterableType<T> {
        pub fn new(
            // _table_name: Identifier,
            column_name: Identifier,
        ) -> Self {
            Self {
                // _table_name,
                column_name,
                _t: PhantomData::default(),
            }
        }
    }

    pub trait FilterEq {
        type Type;
        fn eq(
            &self,
            t: impl Into<<Self as crate::queries::filters::filterable_types::FilterEq>::Type>,
        ) -> Filter;
        fn ne(
            &self,
            t: impl Into<<Self as crate::queries::filters::filterable_types::FilterEq>::Type>,
        ) -> Filter;
    }
    pub trait FilterPartialEq
    where
        Self: FilterEq,
    {
        type Type;
        fn lt(
            &self,
            t: impl Into<<Self as crate::queries::filters::filterable_types::FilterPartialEq>::Type>,
        ) -> Filter;
        fn gt(
            &self,
            t: impl Into<<Self as crate::queries::filters::filterable_types::FilterPartialEq>::Type>,
        ) -> Filter;
        fn lte(
            &self,
            t: impl Into<<Self as crate::queries::filters::filterable_types::FilterPartialEq>::Type>,
        ) -> Filter;
        fn gte(
            &self,
            t: impl Into<<Self as crate::queries::filters::filterable_types::FilterPartialEq>::Type>,
        ) -> Filter;
    }
    pub trait FilterLike {
        type Type;
        fn like(
            &self,
            t: impl Into<<Self as crate::queries::filters::filterable_types::FilterLike>::Type>,
        ) -> Filter;
    }
    pub trait FilterIn {
        type Type;
        fn contained_in(
            &self,
            t: impl Into<<Self as crate::queries::filters::filterable_types::FilterIn>::Type>,
        ) -> Filter;
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

impl AsSql for Filter {
    fn as_sql(&self) -> String {
        type F = Filter;
        match self {
            F::And(children) => {
                if children.len() == 0 {
                    return "true".to_string();
                }
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
            F::Or(children) => {
                if children.len() == 0 {
                    return "".to_string();
                }
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
