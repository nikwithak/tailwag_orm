use std::{ops::Deref, sync::Arc};

use crate::{
    data_definition::table::{DatabaseTableDefinition, Identifier, TableColumn},
    AsSql,
};

#[derive(PartialEq, Eq, Debug, Default)]
pub struct PrimaryKeyColumnConstraint {
    index_parameters: Option<IndexParameters>,
}

impl AsSql for PrimaryKeyColumnConstraint {
    fn as_sql(&self) -> String {
        let mut statement = "PRIMARY KEY".to_string();
        if let Some(params) = &self.index_parameters {
            statement.push(' ');
            statement.push_str(&params.as_sql());
        }
        statement.to_string()
    }
}

struct CheckExpressionConstraint {}

impl AsSql for CheckExpressionConstraint {
    fn as_sql(&self) -> String {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct IndexParameters {}

impl AsSql for IndexParameters {
    fn as_sql(&self) -> String {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct UniqueColumnConstraint {
    is_null_distinct: bool,
    index_parameters: Option<IndexParameters>,
}

impl AsSql for UniqueColumnConstraint {
    fn as_sql(&self) -> String {
        let mut statement = "UNIQUE".to_string();
        if self.is_null_distinct {
            statement.push_str(" NULLS DISTINCT");
        }
        // TODO: index_parameters impl
        statement
    }
}

/// Foreign Key
#[derive(PartialEq, Eq, Debug)]
pub struct ReferencesConstraint {
    pub ref_table: Identifier,
    pub ref_column: Option<Identifier>,
    pub match_type: Option<ReferencesConstraintMatchType>,
    pub on_delete_action: Option<ReferentialAction>,
    pub on_update_action: Option<ReferentialAction>,
}

impl ReferencesConstraint {
    fn new(
        ref_table: Identifier,
        ref_column: Identifier,
    ) -> Self {
        Self {
            ref_table,
            ref_column: Some(ref_column),
            match_type: None,
            on_delete_action: None,
            on_update_action: None,
        }
    }

    fn _compare(
        _old: Self,
        _new: Self,
    ) -> Option<String> {
        todo!()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ReferentialAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull(Vec<Identifier>), // ColumnName - TODO: Find a way to define / enforce this. At least one is required.
    SetDefault(Vec<Identifier>),
}

impl AsSql for ReferentialAction {
    fn as_sql(&self) -> String {
        match self {
            ReferentialAction::NoAction => "NO ACTION".to_string(),
            ReferentialAction::Restrict => "RESTRICT".to_string(),
            ReferentialAction::Cascade => "CASCADE".to_string(),
            ReferentialAction::SetNull(column_names) => {
                let mut statement = "SET NULL ( ".to_string();
                statement.push_str(
                    &column_names
                        .iter()
                        .map(|column_name| column_name.as_str())
                        .collect::<Vec<&str>>()
                        .join(", "),
                );
                statement.push_str(" )");
                statement
            },
            ReferentialAction::SetDefault(column_names) => {
                let mut statement = "SET DEFAULT ( ".to_string();
                statement.push_str(
                    &column_names
                        .iter()
                        .map(|column_name| column_name.as_str())
                        .collect::<Vec<&str>>()
                        .join(", "),
                );
                statement.push_str(" )");
                statement
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ReferencesConstraintMatchType {
    Full,
    Partial,
    Simple,
}

impl AsSql for ReferencesConstraintMatchType {
    fn as_sql(&self) -> String {
        match self {
            ReferencesConstraintMatchType::Full => "MATCH FULL",
            ReferencesConstraintMatchType::Partial => "MATCH PARTIAL",
            ReferencesConstraintMatchType::Simple => "MATCH SIMPLE",
        }
        .to_string()
    }
}

// TODO: Aside from implemetning this, I need to refactor how I handle (some?) constraints that reference other tables, to avoid referencing the table before it exists.

impl AsSql for ReferencesConstraint {
    fn as_sql(&self) -> String {
        let mut statement = format!("REFERENCES {}", &self.ref_table);
        if let Some(column) = &self.ref_column {
            statement.push(' ');
            statement.push_str(column.as_str());
        }
        if let Some(match_type) = &self.match_type {
            statement.push(' ');
            statement.push_str(&match_type.as_sql());
        }
        if let Some(ref_action) = &self.on_delete_action {
            statement.push_str(" ON DELETE ");
            statement.push_str(&ref_action.as_sql());
        }
        if let Some(ref_action) = &self.on_delete_action {
            statement.push_str(" ON UPDATE ");
            statement.push_str(&ref_action.as_sql());
        }

        statement
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TableColumnConstraint {
    pub name: Option<Identifier>,
    pub detail: Arc<TableColumnConstraintDetail>,
}

impl Deref for TableColumnConstraint {
    type Target = TableColumnConstraintDetail;
    fn deref(&self) -> &Self::Target {
        &self.detail
    }
}

impl TableColumnConstraint {
    pub fn new(constraint: TableColumnConstraintDetail) -> Self {
        Self {
            name: None,
            detail: Arc::new(constraint),
        }
    }

    /// Builds a default Unique constraint.
    pub fn unique() -> Self {
        Self {
            name: None,
            detail: Arc::new(
                TableColumnConstraintDetail::Unique(UniqueColumnConstraint::default()),
            ),
        }
    }

    /// Builds a basic foreign key restraint for a column
    pub fn foreign_key<T>(
        ref_table: DatabaseTableDefinition<T>,
        ref_column: TableColumn,
    ) -> Self {
        Self {
            name: None,
            detail: Arc::new(TableColumnConstraintDetail::References(ReferencesConstraint::new(
                ref_table.table_name.clone(),
                ref_column.column_name.clone(),
            ))),
        }
    }

    pub fn primary_key() -> Self {
        Self {
            name: None,
            detail: Arc::new(TableColumnConstraintDetail::PrimaryKey(
                PrimaryKeyColumnConstraint::default(),
            )),
        }
    }

    pub fn non_null() -> Self {
        Self {
            name: None,
            detail: Arc::new(TableColumnConstraintDetail::NotNull),
        }
    }
}

impl AsSql for TableColumnConstraint {
    fn as_sql(&self) -> String {
        if let Some(name) = &self.name {
            format!("CONSTRAINT {} {}", name, &self.detail.as_sql())
        } else {
            self.detail.as_sql()
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum TableColumnConstraintDetail {
    NotNull,
    Null,
    // Check(CheckExpressionConstraint), // TODO
    // Default(_), // TODO
    // Generated(GeneratedConstraint), // TODO
    Unique(UniqueColumnConstraint),
    PrimaryKey(PrimaryKeyColumnConstraint),
    References(ReferencesConstraint),
}

impl AsSql for TableColumnConstraintDetail {
    fn as_sql(&self) -> String {
        match self {
            TableColumnConstraintDetail::NotNull => "NOT NULL".to_owned(),
            TableColumnConstraintDetail::Null => "NULL".to_owned(),
            // TableColumnConstraint::Check(check) => check.as_sql(),
            // TableColumnConstraint::Default(_) => todo!(),
            // TableColumnConstraint::Generated(_) => todo!(),
            TableColumnConstraintDetail::Unique(unique) => unique.as_sql(),
            TableColumnConstraintDetail::PrimaryKey(pk) => pk.as_sql(),
            TableColumnConstraintDetail::References(fk) => fk.as_sql(),
        }
    }
}
