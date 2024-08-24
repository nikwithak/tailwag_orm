pub mod delete;
pub mod insert;
pub mod update;

#[allow(unused)]
pub(crate) trait SqlStatement {
    fn to_sql() -> String;
}
