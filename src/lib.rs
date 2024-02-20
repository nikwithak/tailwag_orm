use sqlx::Postgres;

pub mod data_definition;
pub mod data_manager;
pub mod executor;
pub mod migration;
pub mod object_management;
pub mod queries;

#[derive(Debug)]
pub enum Error {
    Unknown(String),
    Sqlx(sqlx::Error),
    DataIntegrity(String),
    HttpError(reqwest::Error),
    MutexLock(String),
    IoError(std::io::Error),
}
macro_rules! impl_from {
    ($enum:ident::$variant:ident($ty:ty)) => {
        impl From<$ty> for $enum {
            fn from(val: $ty) -> Self {
                Self::$variant(val)
            }
        }
    };
}
impl_from!(Error::Sqlx(sqlx::Error));
impl_from!(Error::IoError(std::io::Error));
impl_from!(Error::Unknown(String));
impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Unknown(val.into())
    }
}
impl_from!(Error::HttpError(reqwest::Error));

// impl<'a, T> From<LockResult<MutexGuard<'a, T>>> for Error {
//     fn from(value: LockResult<MutexGuard<'a, T>>) -> Self {
//         Self::MutexLock(value.to_string())
//     }
// }

// TODO: Move this somewhere else (not lib.rs)
/// Converts the struct into a SQL statement (or component)
pub trait AsSql {
    fn as_sql(&self) -> String;
}

pub trait BuildSql {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    );
}
