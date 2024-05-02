use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Data, DeriveInput, Ident};
use tailwag_utils::macro_utils::attribute_parsing::GetAttribute;

use crate::util::{database_table_definition::get_type_from_field, type_parsing::GetQualifiedPath};

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
    let (create_request_ident, create_request_tokens) = build_create_request(input);

    match &data.fields {
        syn::Fields::Named(fields) => {
            let _field_names = fields.named.iter().map(|f| &f.ident);
            let functions: Vec<TokenStream> = vec![
                // todo!("Add functions here")
                build_get_insert_statement(input),
            ];


            let parse_args_impl_tokens = quote!(
                #create_request_tokens

                impl tailwag::orm::queries::Insertable for #ident
                where
                    Self: tailwag::orm::data_manager::GetTableDefinition, // To make sure error messages show if we can't use `get_table_definition()`
                {
                    type CreateRequest = #create_request_ident;
                    #(#functions)*
                }
            );

            parse_args_impl_tokens
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}

fn build_create_request(input: &DeriveInput) -> (Ident, TokenStream) {
    let Data::Struct(data) = &input.data else {
        panic!("Only Structs are supported")
    };


    // Note: This is generated for build_insert_statement already. If this is slowing down too much, then refactor it to be only called once.
    // Here, it's needed to get the relationship type for child tables.
    let input_table_definition =
        crate::util::database_table_definition::build_table_definition::<()>(input);

    let type_ident = &input.ident;
    let request_ident = format_ident!("{}CreateRequest", input.ident);
    let syn::Fields::Named(fields) = &data.fields else { panic!("Struct contains unnamed fields.")};
    let passthrough_fields = fields.named.iter()
        .filter(|field| field.get_attribute("ignore").is_none())
        .filter(|field|"id" != &field.ident.as_ref().expect("Must have ident on named field").to_string());

    let field_tokens = passthrough_fields.clone().map(
        |field| {
            let (field_name, field_type) = (&field.ident, field.ty.to_token_stream());
            match get_type_from_field(field) {
                tailwag_orm::data_definition::table::DatabaseColumnType::OneToOne(_) => {
                    quote!(pub #field_name: <#field_type as tailwag::orm::queries::Insertable>::CreateRequest,)
                    // todo!()
                },
                _ => quote!(pub #field_name: #field_type,)
            }
            // If field is primitive, pass it through.
            // CURRENT TASK:
            // If field is ONE TO ONE,  TODO
        }
    );
    let field_names = passthrough_fields.clone().map(|field| &field.ident);
    
    (request_ident.clone(), quote!(
        #[derive(Default, serde::Deserialize, serde::Serialize)]
        pub struct #request_ident {
            #(#field_tokens)*
        }
        impl From<#request_ident> for #type_ident {
            fn from(val: #request_ident) -> Self {
                #type_ident {
                    id: uuid::Uuid::new_v4(),
                    #(#field_names: val.#field_names.into(),)*
                    ..Default::default()
                }
            }
        }
    ))
}

fn build_get_insert_statement(input: &DeriveInput) -> TokenStream {
    let input_table_definition =
        crate::util::database_table_definition::build_table_definition::<()>(input);

    let insert_maps = input_table_definition.columns.values().map(|column| {
        let column_name = format_ident!("{}", column.column_name.to_string());
        let mut column_name_as_string = column.column_name.to_string();

        type E = tailwag_orm::data_definition::table::DatabaseColumnType;

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
                column_name_as_string = format!("{column_name}_id");
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
      l          }
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
    let insertable_children: Vec<syn::Ident> = input_table_definition.columns.values().filter_map(|column| {
        let column_name = format_ident!("{}", column.column_name.to_string());

        type E = tailwag_orm::data_definition::table::DatabaseColumnType;

        
        match &column.column_type {
            E::OneToMany(_) => todo!(),
            E::ManyToMany(_) => todo!(),
            E::OneToOne(_) => Some(column_name),
            _ => None,
        }
    }).collect();

    let tokens = quote!(
        fn get_insert_statement(&self) -> Vec<tailwag::orm::object_management::insert::InsertStatement> {
            let mut insert_map = std::collections::HashMap::new();

            #(#insert_maps)*

            let insert = tailwag::orm::object_management::insert::InsertStatement::new(
                <Self as tailwag::orm::data_manager::GetTableDefinition>::get_table_definition().table_name.clone(),
                insert_map,
            );

            let mut transaction_statements = Vec::new();
            #(transaction_statements.append(&mut self.#insertable_children.get_insert_statement());)*
            transaction_statements.push(insert);

            transaction_statements
        }
    );

    tokens
}
