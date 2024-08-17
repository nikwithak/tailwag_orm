mod filters;
pub(crate) mod query_builder;
pub use filters::*;
pub use query_builder::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        // let query = Product::all().filter(create_date.before(DateTime::now()));
    }
}
