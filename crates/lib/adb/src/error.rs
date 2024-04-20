use std::error::Error;
use std::fmt::{Display, Formatter};

/// Error type for the adb crate.
#[derive(Debug)]
pub enum AdbError {
    /// Failed to parse a value.
    Parse {
        value: String,
        source_type: &'static str,
        target_type: &'static str,
        source: Option<Box<dyn Error>>,
    },
}

impl Display for AdbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::io::Error::new(std::io::ErrorKind::Other, "test").source();
        match self {
            Self::Parse {
                value,
                source_type,
                target_type,
                source: reason,
            } => {
                write!(
                    f,
                    "failed when parsing `{}` from `{}` into `{}`",
                    value, source_type, target_type
                )?;
                if let Some(reason) = reason {
                    write!(f, ": {}", reason)
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl Error for AdbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parse { source, .. } => source.as_deref(),
        }
    }
}
