use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Data, DeriveInput, Ident};
use tailwag_utils::macro_utils::attribute_parsing::GetAttribute;

use crate::util::{database_table_definition::get_type_from_field};

/// TODO [TECH DEBT] - The decision to make the SQL queries generated by macros was overkill. When I did this, I had recently started
/// learning to write derive macros, and found this a good chance to practice.
/// 
/// In reality, I should just pull this directly from the DataDefinition. Something like
/// 
/// impl<T> Insert for DataProvider<T> {
///     fn insert(&self, T);
/// }

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
    let (create_request_ident, create_request_tokens) = input.get_attribute("create_type")
        .map(|attr| (attr.parse_args::<Ident>().unwrap(), quote!()))
        .unwrap_or( build_create_request(input));

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

    let type_ident = &input.ident;
    let request_ident = format_ident!("{}CreateRequest", input.ident);
    let syn::Fields::Named(fields) = &data.fields else { panic!("Struct contains unnamed fields.")};
    let passthrough_fields = fields.named.iter()
        .filter(|field| field.get_attribute("db_ignore").is_none() && field.get_attribute("create_ignore").is_none())
        .filter(|field|"id" != &field.ident.as_ref().expect("Must have ident on named field").to_string());

    let field_tokens = passthrough_fields.clone().map(
        |field| {
            let (field_name, field_type) = (&field.ident, field.ty.to_token_stream());
            match get_type_from_field(field) {
                tailwag_orm::data_definition::table::DatabaseColumnType::OneToOne(_) => {
                    quote!(pub #field_name: <#field_type as tailwag::orm::queries::Insertable>::CreateRequest,)
                    // todo!()
                },
                tailwag_orm::data_definition::table::DatabaseColumnType::OneToMany(_) => {
                    quote!(pub #field_name: Vec<<#field_type as tailwag::orm::queries::Insertable>::CreateRequest>,)
                    // todo!()
                },
                _ => quote!(pub #field_name: #field_type,)
            }
            // If field is primitive, pass it through.
        }
    );
    let field_names = passthrough_fields.clone().map(|field| &field.ident);

    // Need to default to any db_ignored fields
    let ignored_fields = fields.named.iter().filter(|field| field.get_attribute("db_ignore").is_some() || field.get_attribute("create_ignore").is_some()).map(|field|&field.ident);

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
                    #(#ignored_fields: Default::default(),)*
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
        let mut field_name = column_name.clone();
        let column_name_as_string = column.column_name.to_string();

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
                field_name = format_ident!("{}", column.column_name.trim_end_matches("_id").to_string()); // Hack to work around soem ugliness with the DataDefinition / column mapping
                // TODO: This assumes ID.
                // column_name_as_string = format!("{column_name}_id");
                quote!(tailwag::orm::data_definition::table::ColumnValue::Uuid(#field_name.id.clone()))
                // todo!()
            },
            E::OneToMany(_) => todo!("{:?} is a OneToMany relationship that isn't yet supported", &column.column_type),
            E::ManyToMany(_) => todo!("{:?} is a ManyToMany relationship that isn't yet supported", &column.column_type),
        };

        
        if column.is_nullable() {
            quote!(
                if let Some(#field_name) = &self.#field_name {
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
                    let #field_name = &self.#field_name;
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
    let insertable_children: Vec<syn::Ident> = input_table_definition.columns.values().filter(|c|!c.is_nullable()).filter_map(|column| {
        // TODO: Same hack as in Updateable. Need to DRY out a lot of this logic.
        let field_name = format_ident!("{}", column.column_name.trim_end_matches("_id").to_string());

        type E = tailwag_orm::data_definition::table::DatabaseColumnType;

        
        match &column.column_type {
            E::OneToMany(_) => todo!(),
            E::ManyToMany(_) => todo!(),
            E::OneToOne(_) => Some(field_name),
            _ => None,
        }
    }).collect();

    // This represents all the optional children - it is needed to wrap the generated code in optional syntax `(.as_ref().map(..))` appropriately.
    let insertable_optional_children: Vec<syn::Ident> = input_table_definition.columns.values().filter(|c|c.is_nullable()).filter_map(|column| {
        // TODO: Same hack as in Updateable. Need to DRY out a lot of this logic.
        let field_name = format_ident!("{}", column.column_name.trim_end_matches("_id").to_string());

        type E = tailwag_orm::data_definition::table::DatabaseColumnType;
        
        match &column.column_type {
            E::OneToMany(_) => todo!(),
            E::ManyToMany(_) => todo!(),
            E::OneToOne(_) => Some(field_name),
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
            #(self.#insertable_optional_children.as_ref().map(|c|transaction_statements.append(&mut c.get_insert_statement()));)*
            transaction_statements.push(insert);

            transaction_statements
        }
    );

    tokens
}
