use super::AdbEnvVar;
use std::env::{self, VarError};

/// `$ADB_MDNS_OPENSCREEN` The default mDNS-SD backend is Bonjour (mdnsResponder).
/// For machines where Bonjour is not installed, adb can spawn its own, embedded, mDNS-SD back end, openscreen.
/// If set to “1”, this env variable forces mDNS backend to openscreen.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AdbMdnsOpenscreen;

impl AdbEnvVar for AdbMdnsOpenscreen {
    type Value = bool;

    const NAME: &'static str = "ADB_MDNS_OPENSCREEN";

    fn get() -> Result<Self::Value, VarError> {
        env::var(Self::NAME).map(|s| s == "1")
    }

    fn set<T: Into<Self::Value>>(var: T) {
        env::set_var(Self::NAME, if var.into() { "1" } else { "0" });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adb::envs::tests::test_env_var_helper;

    #[test]
    fn test_adb_mdns_openscreen() {
        test_env_var_helper::<AdbMdnsOpenscreen>(&[true, false], &[], false);
    }
}
