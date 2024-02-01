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

#[allow(private_bounds)]
pub struct FilterableType<T: TypeFilter> {
    // _table_name: Identifier,
    column_name: Identifier,
    _t: PhantomData<T>,
}

#[allow(private_bounds)]
impl<T: TypeFilter> FilterableType<T> {
    pub fn new(
        // _table_name: Identifier,
        column_name: Identifier,
    ) -> Self {
        Self {
            // _table_name,
            column_name,
            _t: PhantomData,
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
