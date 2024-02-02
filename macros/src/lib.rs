use syn::parse_macro_input;
use tailwag_orm_macro_logic as logic;

#[proc_macro_derive(GetTableDefinition)]
pub fn derive_get_table_definition(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::get_table_definition::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(Deleteable)]
pub fn derive_deleteable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::deleteable::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(Updateable)]
pub fn derive_updateable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::updateable::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(Insertable)]
pub fn derive_insertable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::insertable::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(Id)]
pub fn derive_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::id::derive_struct(&input);
    impl_trait_tokens.into()
}

#[proc_macro_derive(Filterable)]
pub fn derive_filterable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let impl_trait_tokens = logic::derive::filterable::derive_struct(&input);
    impl_trait_tokens.into()
}
