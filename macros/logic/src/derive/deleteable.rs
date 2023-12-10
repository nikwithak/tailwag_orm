use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

fn build_get_delete_statement(_input: &DeriveInput) -> TokenStream {
    // TODO: Delete this if unused? Not sure if I wrote this but hardecoded a hack, or actually just didn't use it.

    // let input_table_definition =
    //     crate::util::database_table_definition::build_table_definition(input);
    //
    // let delete_maps = input_table_definition.columns.iter().map(|column| {
    //     let column_name = format_ident!("{}", column.column_name.as_str());
    //     let column_name_as_string = column.column_name.as_str();
    //
    //     type E = tailwag_orm::data_definition::table::DatabaseColumnType;
    //     let format_string = match &column.column_type {
    //         E::Boolean | E::Int | E::Float => quote!("{}"), // No surrounding quotes - I need to refactor how this works *ANYWAY* (i.e. prep statements)
    //         E::String | E::Timestamp | E::Uuid => quote!(
    //             "'{}'"
    //         ),
    //     };
    //
    //     let delete_statement = if column.is_nullable() {
    //         quote!(
    //
    //             if let Some(#column_name) = &self.#column_name {
    //                 delete_map.insert(
    //                     tailwag::orm::data_definition::table::Identifier::new(#column_name_as_string).expect("Invalid column identifier found - this should not happen. Panicking."),
    //                     format!(#format_string, &#column_name.to_string()),
    //                 );
    //             } else {
    //
    //                 delete_map.insert(
    //                     tailwag::orm::data_definition::table::Identifier::new(#column_name_as_string).expect("Invalid column identifier found - this should not happen. Panicking."),
    //                     "NULL".to_string(),
    //                 );
    //             }
    //         )
    //     } else {
    //         quote!(
    //             delete_map.insert(
    //                 tailwag::orm::data_definition::table::Identifier::new(#column_name_as_string.to_string()).expect("Invalid column identifier found - this should not happen. Panicking."),
    //                 // TODO: Can't support differently named column/struct_attr names right now
    //                 // TODO: Only supports string-like types, apparetly?
    //                 format!(#format_string, &self.#column_name.to_string()),
    //             );
    //         )
    //     };
    //     delete_statement
    // });

    let tokens = quote!(
        fn get_delete_statement(&self) -> tailwag::orm::object_management::delete::DeleteStatement {
            let id_column =
                tailwag::orm::data_definition::table::TableColumn::uuid("id").unwrap().into();
            let id_column = tailwag::orm::queries::FilterComparisonParam::TableColumn(id_column);
            let delete = tailwag::orm::object_management::delete::DeleteStatement::new(
                <Self as tailwag::orm::data_manager::GetTableDefinition>::get_table_definition()
                    .clone(),
                tailwag::orm::queries::Filter::Equal(
                    id_column,
                    tailwag::orm::queries::FilterComparisonParam::Uuid(self.id.clone()),
                ),
            );

            delete
        }
    );

    tokens
}

pub fn derive_struct(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident,
        data,
        ..
    } = &input;

    // Panic with error message if we get a non-struct
    let Data::Struct(data) = data else { panic!("Only Structs are supported") };

    match &data.fields {
        syn::Fields::Named(fields) => {
            let _field_names = fields.named.iter().map(|f| &f.ident);
            let functions: Vec<TokenStream> = vec![
                // todo!("Add functions here")
                build_get_delete_statement(input),
            ];

            let parse_args_impl_tokens = quote!(
                impl tailwag::orm::queries::Deleteable for #ident
                where
                    Self: tailwag::orm::data_manager::GetTableDefinition, // To make sure error messages show if we can't use `get_table_definition()`
                {
                    #(#functions)*
                }
            );

            parse_args_impl_tokens.into()
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}
