//! Operations on adb environment variables.
//!
//! - [`AdbTrace`]: `$ADB_TRACE`, comma (or space) separated list of debug info to log.
//! - [`AdbVendorKeys`]: `$ADB_VENDOR_KEYS`, colon-separated list of keys (files or directories).
//! - [`AndroidSerial`]: `$ANDROID_SERIAL`, serial number to connect to.
//! - [`AndroidLogTags`]: `$ANDROID_LOG_TAGS`, tags to be used by logcat.
//! - [`AdbLocalTransportMaxPort`]: `$ADB_LOCAL_TRANSPORT_MAX_PORT`, max emulator scan port.
//! - [`AdbMdnsAutoConnect`]: `$ADB_MDNS_AUTO_CONNECT`, comma-separated list of mdns services to allow auto-connect.
//! - [`AdbMdnsOpenscreen`]: `$ADB_MDNS_OPENSCREEN`, enable/disable embedded mDNS-SD backend openscreen.
//! - [`AdbLibusb`]: `$ADB_LIBUSB`, enable/disable libusb support.
//!
//! Copied from [ENVIRONMENT VARIABLES](https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/master/docs/user/adb.1.md#environment-variables)

pub mod adb_libusb;
pub mod adb_local_transport_max_port;
pub mod adb_mdns_auto_connect;
pub mod adb_mdns_openscreen;
pub mod adb_trace;
pub mod adb_vendor_keys;
pub mod android_log_tags;
pub mod android_serial;

pub use adb_libusb::AdbLibusb;
pub use adb_local_transport_max_port::AdbLocalTransportMaxPort;
pub use adb_mdns_auto_connect::AdbMdnsAutoConnect;
pub use adb_mdns_openscreen::AdbMdnsOpenscreen;
pub use adb_trace::AdbTrace;
pub use adb_vendor_keys::AdbVendorKeys;
pub use android_log_tags::{AndroidLogTags, FilterSpec, Priority};
pub use android_serial::AndroidSerial;
use std::env::{self, VarError};

/// Environment variable operations.
///
/// # Safety
///
/// [`Self::set`], [`Self::clear`], and [`Self::remove`] are not thread-safe.
/// Usage of these functions in a multi-threaded program should be avoided.
///
/// If [`env::set_var`] or [`env::remove_var`] is marked as unsafe in the future,
/// [`Self::set`], [`Self::clear`], and [`Self::remove`] will follow.
///
/// See [`env::set_var`] for more information.
pub trait AdbEnvVar {
    /// The type of the environment variable.
    type Value;

    /// The name of the environment variable.
    const NAME: &'static str;

    /// The value of the environment variable.
    ///
    /// # Notes
    ///
    /// If the environment variable is not present, but it has a default value according to the
    /// documentation, the return value is [`VarError::NotPresent`].
    /// In this case, you may find a constant named `DEFAULT` or `DEFAULT_*`
    /// in the corresponding implementation.
    ///
    /// # Errors
    ///
    /// - [`VarError::NotPresent`]: The environment variable is not present.
    /// - [`VarError::NotUnicode`]: The value of the environment variable is not valid Unicode.
    fn get() -> Result<Self::Value, VarError>;

    /// Set the value of the environment variable.
    ///
    /// # Safety
    ///
    /// This function is not thread-safe.
    /// Usage of this function in a multi-threaded program should be avoided.
    ///
    /// See [`env::set_var`] for more information.
    fn set<T: Into<Self::Value>>(var: T);

    /// Set the value of the environment variable to an empty string.
    ///
    /// # Safety
    ///
    /// This function is not thread-safe.
    /// Usage of this function in a multi-threaded program should be avoided.
    ///
    /// See [`env::set_var`] for more information.
    fn clear() {
        env::set_var(Self::NAME, "");
    }

    /// Removes the environment variable.
    ///
    /// # Safety
    ///
    /// This function is not thread-safe.
    /// Usage of this function in a multi-threaded program should be avoided.
    ///
    /// See [`env::remove_var`] for more information.
    fn remove() {
        env::remove_var(Self::NAME);
    }
}

#[cfg(test)]
mod tests {
    use super::AdbEnvVar;
    use std::fmt::Debug;

    pub fn test_env_var_helper<Var>(valid: &[Var::Value], invalid: &[Var::Value], clear: Var::Value)
    where
        Var: AdbEnvVar + Debug,
        Var::Value: PartialEq + Debug + Clone,
    {
        // 1. Test if the environment variable is removed.
        Var::remove();
        assert!(Var::get().is_err());
        // 2. Test if the environment variable is set and cleared.
        for value in valid {
            Var::set(value.clone());
            assert_eq!(
                Var::get().unwrap(),
                *value,
                "value: `{:?}`, expect: `{:?}`",
                Var::get().unwrap(),
                *value
            );
            Var::clear();
            assert_eq!(
                Var::get().unwrap(),
                clear,
                "value: `{:?}`, expect: `{:?}`",
                Var::get().unwrap(),
                clear
            );
        }
        Var::remove();
        // 3. Test if invalid values are rejected.
        for value in invalid {
            Var::set(value.clone());
            assert!(Var::get().is_err(), "value: `{:?}`", Var::get().unwrap());
        }
        // 4. Clean up.
        Var::remove();
    }
}
