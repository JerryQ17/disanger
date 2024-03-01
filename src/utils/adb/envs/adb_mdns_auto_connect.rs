use super::AdbEnvVar;
use std::env::{self, VarError};

/// `$ADB_MDNS_AUTO_CONNECT` Comma-separated list of mdns services to allow auto-connect (default adb-tls-connect).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AdbMdnsAutoConnect;

impl AdbEnvVar for AdbMdnsAutoConnect {
    type Value = Vec<String>;

    const NAME: &'static str = "ADB_MDNS_AUTO_CONNECT";

    fn get() -> Result<Self::Value, VarError> {
        env::var(Self::NAME).map(|s| {
            s.split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        })
    }

    fn set<T: Into<Self::Value>>(var: T) {
        env::set_var(Self::NAME, var.into().join(","));
    }
}

impl AdbMdnsAutoConnect {
    pub const DEFAULT: &'static str = "adb-tls-connect";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::adb::envs::tests::test_env_var_helper;

    #[test]
    fn test_adb_mdns_auto_connect() {
        test_env_var_helper::<AdbMdnsAutoConnect>(
            &[vec![
                "adb-tls-connect1".to_string(),
                "adb-tls-connect2".to_string(),
            ]],
            &[],
            vec![],
        );
    }
}
