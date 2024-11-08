use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};
use tailwag_utils::{macro_utils::attribute_parsing::GetAttribute, strings::ToSnakeCase};

pub fn derive_struct(input: &DeriveInput) -> TokenStream {
    let &DeriveInput {
        ident,
        data,
        ..
    } = &input;
    let table_name = format_ident!("{}", ident.to_string().to_snake_case());

    // Panic with error message if we get a non-struct
    let Data::Struct(data) = data else {
        panic!("Only Structs are supported")
    };
    let filter_type_struct_ident = format_ident!("{}Filters", &ident);

    match &data.fields {
        syn::Fields::Named(fields) => {
            let filterable_fields = fields
                .named
                .iter()
                .filter(|field| field.get_attribute("no_filter").is_none())
                .filter(|field| field.get_attribute("db_ignore").is_none());

            let new_fields = filterable_fields.clone().map(|field| {
                let field_ident = field.ident.clone().expect("Should only have named fields.");
                let orig_type = field.ty.clone();
                quote!(pub #field_ident: tailwag::orm::queries::filterable_types::FilterableType<#orig_type>)
            });
            let default_fields = filterable_fields.clone().map(|field| {
                let field_ident = field.ident.clone().expect("Should only have named fields.");
                let field_ident_str = format!("{table_name}.{field_ident}");
                let orig_type = field.ty.clone();
                quote!(#field_ident: tailwag::orm::queries::filterable_types::FilterableType::<#orig_type>::new(tailwag::orm::data_definition::table::Identifier::new_unchecked(#field_ident_str)))
            });

            // OUTPUT STARTS HERE
            quote!(
                pub struct #filter_type_struct_ident {
                    #(#new_fields,)*
                }
                impl Default for #filter_type_struct_ident {
                    fn default() -> Self {
                        Self {
                            #(#default_fields,)*
                        }
                    }
                }
                impl tailwag::orm::queries::filterable_types::Filterable for #ident
                {
                    type FilterType = #filter_type_struct_ident;
                }
            )
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}
