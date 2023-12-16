use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};
use tailwag_utils::strings::to_screaming_snake_case;

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
            let mut struct_name_screaming_case: String =
                to_screaming_snake_case(&ident.to_string());
            struct_name_screaming_case.push_str("_TABLE_DEFINITION");
            let once_cell_name: TokenStream = struct_name_screaming_case.parse().unwrap();

            let functions: Vec<TokenStream> =
                vec![build_get_table_definition(input, once_cell_name.clone())];

            let tokens = quote!(
                static #once_cell_name: std::sync::OnceLock<
                    tailwag::orm::data_definition::table::DatabaseTableDefinition
                > = std::sync::OnceLock::new();
                impl tailwag::orm::data_manager::GetTableDefinition for #ident {
                    #(#functions)*
                }

            );

            tokens.into()
        },
        syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields not supported yet"),
        syn::Fields::Unit => unimplemented!("Unit fields not supported yet"),
    }
}

fn build_get_table_definition(
    input: &DeriveInput,
    once_cell_name: TokenStream,
) -> TokenStream {
    // We build the table definition, and use that to (re-)build the creation logic.
    // Other derives use this same function, so it creates a single place to make changes
    // if the logic on how tables are built changes.
    // Additional changes to support any syntax changes must still be added here

    // [2023-12-11] TODO: I'm gonna have to rethink this piece. The current way we build table defs is making
    // it hard to do one-to-manies in the way I want, because it relies on a child table having references.
    let input_table_definition =
        crate::util::database_table_definition::build_table_definition(input);

    // Build columns
    let table_columns = input_table_definition.columns.iter().map(|(_, column)| {
        let column_name: &str = &column.column_name;
        let column_type = match &column.column_type {
            tailwag_orm::data_definition::table::DatabaseColumnType::Boolean=>quote!(tailwag::orm::data_definition::table::DatabaseColumnType::Boolean),
            tailwag_orm::data_definition::table::DatabaseColumnType::Int=>quote!(tailwag::orm::data_definition::table::DatabaseColumnType::Int),
            tailwag_orm::data_definition::table::DatabaseColumnType::Float=>quote!(tailwag::orm::data_definition::table::DatabaseColumnType::Float),
            tailwag_orm::data_definition::table::DatabaseColumnType::String=>quote!(tailwag::orm::data_definition::table::DatabaseColumnType::String),
            tailwag_orm::data_definition::table::DatabaseColumnType::Timestamp=>quote!(tailwag::orm::data_definition::table::DatabaseColumnType::Timestamp),
            tailwag_orm::data_definition::table::DatabaseColumnType::Uuid=>quote!(tailwag::orm::data_definition::table::DatabaseColumnType::Uuid),
            tailwag_orm::data_definition::table::DatabaseColumnType::Json=>quote!(tailwag::orm::data_definition::table::DatabaseColumnType::Json),
            tailwag_orm::data_definition::table::DatabaseColumnType::OneToMany(child) => {
                let child = &***child;
                quote!(tailwag::orm::data_definition::table::DatabaseColumnType::OneToMany(Identifier::new(#child)).unwrap())
            }
            tailwag_orm::data_definition::table::DatabaseColumnType::ManyToMany(child) => {
                let child = &***child;
                quote!(tailwag::orm::data_definition::table::DatabaseColumnType::ManyToMany(Identifier::new(#child)).unwrap())
            }
            tailwag_orm::data_definition::table::DatabaseColumnType::OneToOne(child) => {
                let child = &***child;
                quote!(tailwag::orm::data_definition::table::DatabaseColumnType::OneToOne(Identifier::new(#child)).unwrap())
            }
        };
        let constraints = column.constraints.iter().map(|constraint| {
            match *constraint.detail {
                tailwag_orm::data_definition::table::TableColumnConstraintDetail::NotNull => quote!(.non_null()),
                tailwag_orm::data_definition::table::TableColumnConstraintDetail::PrimaryKey(_) => quote!(.pk()),
                tailwag_orm::data_definition::table::TableColumnConstraintDetail::References(_) => todo!(),
                tailwag_orm::data_definition::table::TableColumnConstraintDetail::Unique(_) => todo!(),
                tailwag_orm::data_definition::table::TableColumnConstraintDetail::Null => quote!(),
            }
        });

        quote!(
            tailwag::orm::data_definition::table::TableColumn::new(&#column_name, #column_type, Vec::new()).expect("Invalid column name")
            #(#constraints)*
        )
    });

    let table_name = input_table_definition.table_name.as_str();

    // !! START OF QUOTE
    let tokens = quote!(
        fn get_table_definition() -> tailwag::orm::data_definition::table::DatabaseTableDefinition {
            let table_def:  &tailwag::orm::data_definition::table::DatabaseTableDefinition = #once_cell_name.get_or_init(|| {
                tailwag::orm::data_definition::table::DatabaseTableDefinition::new(&#table_name)
                    .expect("Table name is invalid")
                    #(.column(#table_columns))*
                    // #(.constraint(#table_constraints)*) // TODO - weak constriants support currently
                    .into()
            });

            table_def.clone()
        }
    );
    // !! END OF QUOTE

    tokens
}
