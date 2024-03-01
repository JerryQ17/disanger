use super::AdbEnvVar;
use std::env::{self, VarError};

/// `$ADB_LIBUSB` ADB has its own USB backend implementation but can also employ libusb.
///
/// Use below commands to identify which is in use:
/// - `adb devices -l` (usb: prefix is omitted for libusb)
/// - `adb host-features` (look for libusb in the output list)
///
/// To override the default for your OS, set `$ADB_LIBUSB` to
/// - "1" to enable libusb
/// - "0" to enable the ADB backend implementation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AdbLibusb;

impl AdbEnvVar for AdbLibusb {
    type Value = bool;

    const NAME: &'static str = "ADB_LIBUSB";

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
    use crate::utils::adb::envs::tests::test_env_var_helper;

    #[test]
    fn test_adb_libusb() {
        test_env_var_helper::<AdbLibusb>(&[true, false], &[], false);
    }
}
