pub mod database_definition;
pub mod executor;
pub mod queries;

pub trait AsSql {
    fn as_sql(&self) -> String;
}
