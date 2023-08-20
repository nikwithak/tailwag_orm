use std::{ops::Deref, sync::Arc};

use crate::{database_definition::table_definition::Identifier, AsSql};

#[derive(PartialEq, Eq, Debug)]
pub struct PrimaryKeyConstraint {
    index_parameters: Option<IndexParameters>,
}

impl AsSql for PrimaryKeyConstraint {
    fn as_sql(&self) -> String {
        let mut statement = "PRIMARY KEY".to_string();
        if let Some(params) = &self.index_parameters {
            statement.push_str(" ");
            statement.push_str(&params.as_sql());
        }
        statement.to_string()
    }
}

impl Default for PrimaryKeyConstraint {
    fn default() -> Self {
        PrimaryKeyConstraint {
            index_parameters: None,
        }
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

#[derive(PartialEq, Eq, Debug)]
pub struct UniqueConstraint {
    is_null_distinct: bool,
    index_parameters: Option<IndexParameters>,
}
impl Default for UniqueConstraint {
    fn default() -> Self {
        Self {
            is_null_distinct: false,
            index_parameters: None,
        }
    }
}

impl AsSql for UniqueConstraint {
    fn as_sql(&self) -> String {
        let mut statement = "UNIQUE".to_string();
        if self.is_null_distinct {
            statement.push_str(" NULLS DISTINCT");
        }
        statement
    }
}

/// Foreign Key
#[derive(PartialEq, Eq, Debug)]
pub struct ReferencesConstraint {
    ref_table: Identifier,
    ref_column: Option<Identifier>,
    match_type: Option<ReferencesConstraintMatchType>,
    on_delete_action: Option<ReferentialAction>,
    on_update_action: Option<ReferentialAction>,
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
}

#[derive(PartialEq, Eq, Debug)]
enum ReferentialAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull(Vec<Identifier>), // ColumnName - TODO: Find a way to define / enforce this. At least one is required.
    SetDefault(Vec<Identifier>),
}

#[derive(PartialEq, Eq, Debug)]
enum ReferencesConstraintMatchType {
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

impl AsSql for ReferencesConstraint {
    fn as_sql(&self) -> String {
        todo!()
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
        return &self.detail;
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
            detail: Arc::new(TableColumnConstraintDetail::Unique(UniqueConstraint::default())),
        }
    }

    /// Builds a basic foreign key restraint for a column
    pub fn foreign_key(
        ref_table: Identifier,
        ref_column: Identifier,
    ) -> Self {
        Self {
            name: None,
            detail: Arc::new(TableColumnConstraintDetail::References(ReferencesConstraint::new(
                ref_table,
                ref_column,
            ))),
        }
    }

    pub fn primary_key() -> Self {
        Self {
            name: None,
            detail: Arc::new(TableColumnConstraintDetail::PrimaryKey(
                PrimaryKeyConstraint::default(),
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
    Unique(UniqueConstraint),
    PrimaryKey(PrimaryKeyConstraint),
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
