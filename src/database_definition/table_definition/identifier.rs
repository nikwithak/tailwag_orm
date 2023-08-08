use std::ops::Deref;

/// Represents a Database Identifier, for column names and table names.
/// It's just a string under the hood, but forcing calls to use Identifier::new(String),
/// we are able to perform field validation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Identifier {
    value: String,
}

impl Deref for Identifier {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return self.value.fmt(f);
    }
}

impl Identifier {
    pub fn new<S: Into<String>>(value: S) -> Result<Self, &'static str> {
        let value: String = value.into();
        if value.chars().all(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => true,
            _ => false,
        }) {
            Ok(Self {
                value,
            })
        } else {
            Err("Contains invalid characters. [a-zA-Z0-9_] are only allowed values.")
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
