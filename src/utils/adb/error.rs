use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub struct OptionParseError {
    message: String,
}

impl OptionParseError {
    pub fn new(value: String, target: &'static str) -> Self {
        Self {
            message: format!("cannot parse `{}` into type `{}`", value, target),
        }
    }

    pub fn with_reason<V, T, R>(value: V, target: T, reason: R) -> Self
    where
        V: Display,
        T: Display,
        R: Display,
    {
        Self {
            message: format!(
                "cannot parse `{}` into type `{}`: {}",
                value, target, reason
            ),
        }
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl Display for OptionParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for OptionParseError {}

impl From<String> for OptionParseError {
    fn from(value: String) -> Self {
        Self { message: value }
    }
}

impl From<&str> for OptionParseError {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}
