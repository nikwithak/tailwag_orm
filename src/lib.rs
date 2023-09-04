pub mod database_definition;
pub mod executor;
pub mod migration;
pub mod object_management;
pub mod queries;

// TODO: Move this somewhere else (not lib.rs)
/// Converts the struct into a SQL statement (or component)
pub trait AsSql {
    fn as_sql(&self) -> String;
}

pub trait FromSql {
    fn from_sql<T: AsSql>(_sql: &str) -> T {
        todo!("FromSql is not yet implemented.")
    }
}
