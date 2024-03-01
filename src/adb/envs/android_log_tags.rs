use super::AdbEnvVar;
use std::env::{self, VarError};
use std::fmt::Display;
use std::str::FromStr;

/// The priority of the log message.
/// - [`Priority::Verbose`]: V    Verbose (default for `<tag>`)
/// - [`Priority::Debug`]: D    Debug (default for `*`)
/// - [`Priority::Info`]: I    Info
/// - [`Priority::Warn`]: W    Warn
/// - [`Priority::Error`]: E    Error
/// - [`Priority::Fatal`]: F    Fatal
/// - [`Priority::Silent`]: S    Silent (suppress all output)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Priority {
    #[default]
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Silent,
}

impl Priority {
    pub const VERBOSE: &'static str = "V";
    pub const DEBUG: &'static str = "D";
    pub const INFO: &'static str = "I";
    pub const WARN: &'static str = "W";
    pub const ERROR: &'static str = "E";
    pub const FATAL: &'static str = "F";
    pub const SILENT: &'static str = "S";

    pub const fn as_str(&self) -> &str {
        match self {
            Self::Verbose => Self::VERBOSE,
            Self::Debug => Self::DEBUG,
            Self::Info => Self::INFO,
            Self::Warn => Self::WARN,
            Self::Error => Self::ERROR,
            Self::Fatal => Self::FATAL,
            Self::Silent => Self::SILENT,
        }
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Priority::VERBOSE => Ok(Priority::Verbose),
            Priority::DEBUG => Ok(Priority::Debug),
            Priority::INFO => Ok(Priority::Info),
            Priority::WARN => Ok(Priority::Warn),
            Priority::ERROR => Ok(Priority::Error),
            Priority::FATAL => Ok(Priority::Fatal),
            Priority::SILENT => Ok(Priority::Silent),
            _ => Err(format!("Invalid log priority value: {}", s)),
        }
    }
}

/// FilterSpecs are a series of `<tag>[:priority]` where `<tag>` is a
/// log component tag (or `*` for all) and priority is a variant of [`Priority`].
/// `*` by itself means `*:D` and `<tag>` by itself means `<tag>:V`.
///
///  # Equivalent form
///
/// `<tag>:<priority>`
///
/// ```
/// use disanger::adb::envs::{FilterSpec, Priority};
///
/// assert_eq!(FilterSpec::default().to_string(), "*:V");
/// assert_eq!(
///     FilterSpec {
///         tag: "test".to_string(),
///         priority: Priority::Debug,
///     }.to_string(),
///     "test:D"
/// );
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterSpec {
    pub tag: String,
    pub priority: Priority,
}

impl Display for FilterSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.tag, self.priority)
    }
}

impl FromStr for FilterSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tag, priority) = if s == "*" {
            //`*` by itself means `*:D`
            ("*", Priority::Debug)
        } else if !s.contains(':') {
            //`<tag>` by itself means `<tag>:V`
            (s, Priority::Verbose)
        } else {
            let (t, p) = s
                .split_once(':')
                .filter(|(t, p)| !t.is_empty() && !p.is_empty())
                .ok_or_else(|| format!("Invalid filter spec: {s}"))?;
            (t, p.parse()?)
        };
        Ok(Self {
            tag: tag.to_string(),
            priority,
        })
    }
}

impl Default for FilterSpec {
    fn default() -> Self {
        Self {
            tag: "*".to_string(),
            priority: Priority::Verbose,
        }
    }
}

/// `$ANDROID_LOG_TAGS`: Tags to be used by logcat.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AndroidLogTags;

impl AdbEnvVar for AndroidLogTags {
    type Value = Vec<FilterSpec>;

    const NAME: &'static str = "ANDROID_LOG_TAGS";

    fn get() -> Result<Self::Value, VarError> {
        Ok(env::var(Self::NAME)?
            .split(' ')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap_or_default())
            .collect())
    }

    fn set<T: Into<Self::Value>>(var: T) {
        env::set_var(
            Self::NAME,
            var.into()
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adb::envs::tests::test_env_var_helper;

    #[test]
    fn test_filter_spec() {
        let oks = [
            (
                "test1:D",
                FilterSpec {
                    tag: "test1".to_string(),
                    priority: Priority::Debug,
                },
            ),
            (
                "test2:V",
                FilterSpec {
                    tag: "test2".to_string(),
                    priority: Priority::Verbose,
                },
            ),
            (
                "test3:I",
                FilterSpec {
                    tag: "test3".to_string(),
                    priority: Priority::Info,
                },
            ),
            (
                "test4:W",
                FilterSpec {
                    tag: "test4".to_string(),
                    priority: Priority::Warn,
                },
            ),
            (
                "test5:E",
                FilterSpec {
                    tag: "test5".to_string(),
                    priority: Priority::Error,
                },
            ),
            (
                "test6:F",
                FilterSpec {
                    tag: "test6".to_string(),
                    priority: Priority::Fatal,
                },
            ),
            (
                "test7:S",
                FilterSpec {
                    tag: "test7".to_string(),
                    priority: Priority::Silent,
                },
            ),
            (
                "*",
                FilterSpec {
                    tag: "*".to_string(),
                    priority: Priority::Debug,
                },
            ),
            (
                "test8",
                FilterSpec {
                    tag: "test8".to_string(),
                    priority: Priority::Verbose,
                },
            ),
        ];
        for (input, expected) in oks {
            assert_eq!(input.parse::<FilterSpec>().unwrap(), expected);
        }
        let errs = ["test1:A", ":", "test3:", ":V"];
        for input in errs {
            assert!(input.parse::<FilterSpec>().is_err());
        }
    }

    #[test]
    fn test_android_log_tags() {
        test_env_var_helper::<AndroidLogTags>(
            &[vec![
                FilterSpec {
                    tag: "test1".to_string(),
                    priority: Priority::Debug,
                },
                FilterSpec {
                    tag: "test2".to_string(),
                    priority: Priority::Verbose,
                },
            ]],
            &[],
            vec![],
        );
    }
}
