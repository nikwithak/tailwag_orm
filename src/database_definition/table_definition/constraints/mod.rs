use super::{IndexParameters, TableColumn};

#[derive(PartialEq, Eq, Debug)]
pub enum TableConstraintDetail {
    Unique(UniqueConstraint),
    PrimaryKey(),
    ForeignKey(),
}

#[derive(PartialEq, Eq, Debug)]
pub struct UniqueConstraint {
    is_null_distinct: bool,
    index_parameterse: Option<IndexParameters>,
    columns: Vec<TableColumn>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct PrimaryKeyConstraint {
    index_parameters: Option<IndexParameters>,
    columns: Vec<TableColumn>,
    
}
