pub mod delete;
pub mod insert;
pub mod update;
pub(crate) trait SqlStatement {
    fn to_sql() -> String;
}
