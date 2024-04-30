use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

fn build_get_insert_statement(input: &DeriveInput) -> TokenStream {
    let input_table_definition =
        crate::util::database_table_definition::build_table_definition::<()>(input);

    let insert_maps = input_table_definition.columns.values().map(|column| {
        let column_name = format_ident!("{}", column.column_name.to_string());
        let column_name_as_string = column.column_name.to_string();

        type E = tailwag_orm::data_definition::table::DatabaseColumnType;
        mod brainstorm
        {
            // If we find a CHILD object that we need to insert, we need to pre-insert it.
            // Do that with ""

        }
        let wrapped_type = match &column.column_type {
            E::Boolean => quote!(tailwag::orm::data_definition::table::ColumnValue::Boolean(#column_name.clone())),
            E::Int => quote!(tailwag::orm::data_definition::table::ColumnValue::Int(#column_name.clone())),
            E::Float => quote!(tailwag::orm::data_definition::table::ColumnValue::Float(#column_name.clone())),
            E::String => quote!(tailwag::orm::data_definition::table::ColumnValue::String(#column_name.to_string())),
            E::Timestamp => quote!(tailwag::orm::data_definition::table::ColumnValue::Timestamp(#column_name.clone())),
            E::Uuid => quote!(tailwag::orm::data_definition::table::ColumnValue::Uuid(#column_name.clone())),
            E::Json => quote!(tailwag::orm::data_definition::table::ColumnValue::Json(#column_name.to_string())),
            E::OneToOne(_child_type) => {
                // TODO: This assumes ID.
                // let column_name = format_ident!("{}_id", &column_name);
                dbg!(quote!(tailwag::orm::data_definition::table::ColumnValue::Uuid(#column_name.id.clone())))
                // todo!()
            },
            E::OneToMany(_) => todo!("{:?} is a OneToMany relationship that isn't yet supported", &column.column_type),
            E::ManyToMany(_) => todo!("{:?} is a ManyToMany relationship that isn't yet supported", &column.column_type),
        };

        
        if column.is_nullable() {
            quote!(
                if let Some(#column_name) = &self.#column_name {
                    insert_map.insert(
                        tailwag::orm::data_definition::table::Identifier::new(#column_name_as_string)
                        .expect("Invalid column identifier found - this should not happen."),
                        #wrapped_type,
                    );
                }
                // TODO - else insert null, explicitly?
            )
        } else {
            quote!(
                {
                    let #column_name = &self.#column_name;
                    insert_map.insert(
                        tailwag::orm::data_definition::table::Identifier::new(#column_name_as_string.to_string()).expect("Invalid column identifier found - this should not happen. Panicking."),
                        // TODO: Can't support differently named column/struct_attr names right now
                        // TODO: Only supports string-like types, apparetly?
                        #wrapped_type,
                    );
                }
            )
        }
    });

    let tokens = quote!(
        fn get_insert_statement(&self) -> tailwag::orm::object_management::insert::InsertStatement {
            let mut insert_map = std::collections::HashMap::new();

            #(#insert_maps)*

            let insert = tailwag::orm::object_management::insert::InsertStatement::new(
                <Self as tailwag::orm::data_manager::GetTableDefinition>::get_table_definition().table_name.clone(),
                insert_map,
            );
            insert
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
    let Data::Struct(data) = data else {
        panic!("Only Structs are supported")
    };

    match &data.fields {
        syn::Fields::Named(fields) => {
            let _field_names = fields.named.iter().map(|f| &f.ident);
            let functions: Vec<TokenStream> = vec![
                // todo!("Add functions here")
                build_get_insert_statement(input),
            ];

            let parse_args_impl_tokens = quote!(
                impl tailwag::orm::queries::Insertable for #ident
                where
                    Self: tailwag::orm::data_manager::GetTableDefinition, // To make sure error messages show if we can't use `get_table_definition()`
                {
                    #(#functions)*
                }
            );

            parse_args_impl_tokens
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}
