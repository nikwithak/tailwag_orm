use std::marker::PhantomData;

use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::data_definition::table::{Identifier, TableColumn};

use super::Filter;

pub trait Filterable {
    type FilterType: Default;
}
// Trying out the type-state pattern here.
trait TypeFilter {}
macro_rules! typetype {
    ($item:ty) => {
        // pub enum $item {}
        impl TypeFilter for $item {}
    };
}
macro_rules! impl_filter_for {
        ($item:ty: $base_type:ty, $table_column_fn_name:ident, $param_type_enum:ident, $trait_name:ident $($func_name:ident:$comparison_type:ident),*) => {
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

macro_rules! impl_numeric_type {
    ($type:ty: $db_type:ident) => {
        typetype! {$type}
        impl_filter_for!($type: $type, new_int, $db_type, FilterEq eq:Equal, ne:NotEqual);
        impl_filter_for!($type: $type, new_int, $db_type, FilterPartialEq lt:LessThan, lte:LessThanOrEqual, gt:GreaterThan, gte:GreaterThanOrEqual);
    }
}

// Here we define the filterabilities of each type
typetype! {Uuid}
impl_filter_for!(Uuid: uuid::Uuid, new_uuid, Uuid, FilterEq eq:Equal, ne:NotEqual);
impl_filter_for!(Uuid: uuid::Uuid, new_uuid, Uuid, FilterLike like:Like);
typetype! {bool}
impl_filter_for!(bool: bool, new_bool, Bool, FilterEq eq:Equal, ne:NotEqual);
typetype! {String}
impl_filter_for!(String: String, new_string, String, FilterEq eq:Equal, ne:NotEqual);
impl_filter_for!(String: String, new_string, String, FilterPartialEq lt:LessThan, lte:LessThanOrEqual, gt:GreaterThan, gte:GreaterThanOrEqual);
impl_filter_for!(String: String, new_string, String, FilterLike like:Like);
impl_numeric_type!(i64: Integer);
impl_numeric_type!(f64: Float);
// TODO: Implement this for stronger dynamic typing with numerics
// impl_numeric_type!(usize: Integer);
// impl_numeric_type!(u64: Integer);
// impl_numeric_type!(u32: Integer);
// impl_numeric_type!(u16: Integer);
// impl_numeric_type!(u8: Integer);
// impl_numeric_type!(isize: Integer);
// impl_numeric_type!(i32: Integer);
// impl_numeric_type!(i16: Integer);
// impl_numeric_type!(i8: Integer);
// impl_numeric_type!(f32: Float);
typetype! {chrono::NaiveDateTime}
impl_filter_for!(chrono::NaiveDateTime: chrono::NaiveDateTime, new_timestamp, Timestamp, FilterEq eq:Equal, ne:NotEqual);

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
