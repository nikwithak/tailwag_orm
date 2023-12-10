/// TODO: Move the contents of this file outside, into a macro logic crate.
///
/// That was the original point of this crate, but it has evolved into being used as ORM.
use syn::{Data, DeriveInput, Field, GenericArgument, PathArguments};

use tailwag_orm::data_definition::table::{
    DatabaseColumnType, DatabaseTableDefinition, TableColumn,
};

use super::attribute_parsing::GetAttribute;

pub(crate) fn build_table_definition(input: &DeriveInput) -> DatabaseTableDefinition {
    let &DeriveInput {
        ident,
        data,
        ..
    } = &input;
    // TODO: Force this to snake_case from CapsCase
    let table_name = tailwag_utils::strings::to_snake_case(&ident.to_string());

    // Panic with error message if we get a non-struct
    let Data::Struct(data) = data else { panic!("Only Structs are supported.") };
    let syn::Fields::Named(fields) = &data.fields else { panic!("Unnamed fields found in the struct.")};

    let columns = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().expect("Found unnamed field in struct");

        let mut column =
            TableColumn::new(&field_name.to_string(), get_type_from_field(&f), Vec::new())
                .expect("Invalid table_name");
        // TODO: Handle #[flatten], which will flatten the pieces into a single table. Will that work? Gonna be tough in a derive macro.

        if &field_name.to_string() == "id" || f.get_attribute("primary_key").is_some() {
            column = column.pk();
        }

        if !is_option(&f) {
            column = column.non_null();
        }

        // TODO: If _??!!??!???? then handle foreign keys
        column
    });

    let mut table = DatabaseTableDefinition::new(&table_name).expect("Table name is invalid");
    for column in columns {
        table.add_column(column);
    }

    table.into()
}

// Re-exports for backwards compatility for paths already using these. Will be removed in the future.
pub use super::type_parsing::get_qualified_path;
pub use super::type_parsing::is_option;

pub fn get_type_from_field(field: &Field) -> DatabaseColumnType {
    match &field.ty {
        syn::Type::Path(typepath) => {
            // Match the type - if it's a supported type, we map it to the DatabaseColumnType. If it's not, we either fail (MVP), or we add support for joins via another trait (must impl DatabaseColumnSubType or something).
            // TODO: DRY this out using the `is_option` fn above
            let mut qualified_path = get_qualified_path(typepath);
            qualified_path = match qualified_path.as_str() {
                "std::option::Option" | "core::option::Option" | "option::Option" | "Option" => {
                    let type_params = &typepath
                        .path
                        .segments
                        .last()
                        .expect("Option should have an inner type")
                        .arguments;
                    match &type_params {
                        PathArguments::AngleBracketed(params) => {
                            let arg = params.args.first().expect("No type T found for Option<T>");
                            match arg {
                                GenericArgument::Type(syn::Type::Path(t)) => {
                                    Some(get_qualified_path(t))
                                },
                                _ => panic!("no type T found for Option<T>"),
                            }
                        },
                        _ => panic!("No type T found for Option<T>"),
                    }
                },
                _ => None,
            }
            .unwrap_or(qualified_path);

            let db_type = if let Some(_) = field.get_attribute("string") {
                DatabaseColumnType::String
            } else {
                match qualified_path.as_str() {
                    "std::string::String" | "string::String" | "String" => {
                        DatabaseColumnType::String
                    },
                    "bool" => DatabaseColumnType::Boolean,
                    "u32" | "u64" | "i32" | "i64" | "usize" | "isize" => DatabaseColumnType::Int,
                    "f32" | "f64" | "fsize" => DatabaseColumnType::Float,
                    "chrono::_" => DatabaseColumnType::Timestamp, // TODO
                    "uuid::Uuid" | "Uuid" => DatabaseColumnType::Uuid,
                    _ => {
                        // TODO: Clean this up / make it more dynamic.
                        // DatabaseColumnType::Json
                        DatabaseColumnType::String
                    },
                }
            };
            db_type
        },
        _ => {
            unimplemented!("Not a supported data type")
        },
    }
}
