use snafu::{ensure, Snafu};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonPointer {
    components: Vec<String>,
}

impl JsonPointer {
    pub fn components(&self) -> Vec<&str> {
        self.components.iter().map(|x| x.as_str()).collect()
    }
}

#[derive(Debug, Clone, Snafu)]
pub enum ParseJsonPointerError {
    #[snafu(display(
        "Only relative pointers supported, but `{}` doesn't start with a `#`",
        original
    ))]
    NotRelative { original: String },
}

impl FromStr for JsonPointer {
    type Err = ParseJsonPointerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(s.starts_with("#/"), NotRelative { original: s.to_string() });
        Ok(JsonPointer {
            components: s.trim_start_matches("#/").split("/").map(|x| x.to_string()).collect(),
        })
    }
}
