use super::AdbEnvVar;
use std::env::{self, VarError};
use std::path::PathBuf;

/// `$ADB_VENDOR_KEYS`: Colon-separated list of keys (files or directories).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AdbVendorKeys;

impl AdbEnvVar for AdbVendorKeys {
    type Value = Vec<PathBuf>;

    const NAME: &'static str = "ADB_VENDOR_KEYS";

    fn get() -> Result<Self::Value, VarError> {
        Ok(env::var(Self::NAME)?
            .split(':')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect::<Vec<_>>())
    }

    fn set<T: Into<Self::Value>>(var: T) {
        env::set_var(
            Self::NAME,
            var.into()
                .iter()
                .map(|p| p.as_os_str())
                .collect::<Vec<_>>()
                .join(":".as_ref()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adb::envs::tests::test_env_var_helper;

    #[test]
    fn test_adb_vendor_keys() {
        test_env_var_helper::<AdbVendorKeys>(
            &[
                vec![PathBuf::from("/path/to/key")],
                vec![
                    PathBuf::from("/path/to/key1"),
                    PathBuf::from("/path/to/key2"),
                ],
            ],
            &[],
            vec![],
        );
    }
}
