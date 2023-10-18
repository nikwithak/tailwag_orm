use std::sync::Arc;

use crate::AsSql;

use super::{
    Identifier, IndexParameters, ReferencesConstraintMatchType, ReferentialAction, TableColumn,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TableConstraint {
    pub name: Option<Identifier>,
    pub detail: Arc<TableConstraintDetail>,
}

impl AsSql for TableConstraint {
    fn as_sql(&self) -> String {
        if let Some(name) = &self.name {
            format!("CONSTRAINT {} {}", name, self.detail.as_sql())
        } else {
            self.detail.as_sql()
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum TableConstraintDetail {
    Unique(UniqueConstraint),
    PrimaryKey(PrimaryKeyConstraint),
    ForeignKey(ForeignKeyConstraint),
}

impl AsSql for TableConstraintDetail {
    fn as_sql(&self) -> String {
        match self {
            Self::ForeignKey(_fk) => "",
            Self::Unique(_) => todo!(),
            Self::PrimaryKey(_) => todo!(),
        }
        .into()
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct UniqueConstraint {
    is_null_distinct: bool,
    index_parameters: Option<IndexParameters>,
    columns: Vec<TableColumn>,
}

impl AsSql for UniqueConstraint {
    fn as_sql(&self) -> String {
        let mut statement = "UNIQUE".to_string();
        if self.is_null_distinct {
            statement.push_str(" NULLS DISTINCT");
        }

        // Collect the column names
        let columns = self
            .columns
            .iter()
            .map(|col| col.column_name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
        statement.push_str(" ");
        statement.push_str(&columns);

        // TODO: index_parameters impl
        statement
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct PrimaryKeyConstraint {
    index_parameters: Option<IndexParameters>,
    columns: Vec<TableColumn>,
}

impl AsSql for PrimaryKeyConstraint {
    fn as_sql(&self) -> String {
        let mut sql = "PRIMARY KEY ".to_string();

        // Collect the column names
        let columns = self
            .columns
            .iter()
            .map(|col| col.column_name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
        sql.push_str(&columns);
        sql
        // TODO: IndexParameters impl
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct ForeignKeyConstraint {
    pub ref_table: Identifier,
    pub ref_columns: Vec<Identifier>, // TODO: Yank this to a more definitive mapping from columns and ref_columns
    pub columns: Vec<TableColumn>,
    pub match_type: Option<ReferencesConstraintMatchType>,
    pub on_delete_action: Option<ReferentialAction>,
    pub on_update_action: Option<ReferentialAction>,
}
