mod postgres;
pub use postgres::*;
pub mod rest_api;
pub mod threaded;
pub mod traits;

pub mod local_files;
// struct PostgresDataProvider {}
// struct FileS3DataProvider {}
// struct FileLocalStorageProvider {}
// struct MongoDBDataProvider {}
