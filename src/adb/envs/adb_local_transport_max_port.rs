use super::AdbEnvVar;
use std::env::{self, VarError};

/// `$ADB_LOCAL_TRANSPORT_MAX_PORT` Max emulator scan port (default 5585, 16 emulators).
///
/// Note that [`AdbLocalTransportMaxPort::set`] and [`AdbLocalTransportMaxPort::clear`]
/// may behave not as what you expect.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AdbLocalTransportMaxPort;

impl AdbEnvVar for AdbLocalTransportMaxPort {
    type Value = u16;

    const NAME: &'static str = "ADB_LOCAL_TRANSPORT_MAX_PORT";

    /// The value of `$ADB_LOCAL_TRANSPORT_MAX_PORT`.
    ///
    /// # Errors
    ///
    /// Note that [`VarError::NotUnicode`] here has different meaning from the original.
    ///
    /// - [`VarError::NotPresent`]: The environment variable is not present.
    /// - [`VarError::NotUnicode`]: The value of the environment variable cannot be parsed as **a valid port number**.
    fn get() -> Result<Self::Value, VarError> {
        env::var(Self::NAME).and_then(|s| s.parse().map_err(|_| VarError::NotUnicode(s.into())))
    }

    /// Set the value of `$ADB_LOCAL_TRANSPORT_MAX_PORT`.
    ///
    /// `var` should be greater than or equal to [`Self::DEFAULT_MIN_PORT`] (5555),
    /// otherwise, it will be set to [`Self::DEFAULT_MAX_PORT`] (5585).
    ///
    /// # Safety
    ///
    /// This function is not thread-safe.
    /// Usage of this function in a multi-threaded program should be avoided.
    ///
    /// See [`env::set_var`] for more information.
    fn set<T: Into<Self::Value>>(var: T) {
        let mut var = var.into();
        if var < Self::DEFAULT_MIN_PORT {
            var = Self::DEFAULT_MAX_PORT;
        }
        env::set_var(Self::NAME, var.to_string());
    }

    /// Set the value of the environment variable to [`Self::DEFAULT_MAX_PORT`] (5585),
    /// **not** an empty string.
    ///
    /// # Safety
    ///
    /// This function is not thread-safe.
    /// Usage of this function in a multi-threaded program should be avoided.
    ///
    /// See [`env::set_var`] for more information.
    fn clear() {
        Self::set(Self::DEFAULT_MAX_PORT);
    }
}

impl AdbLocalTransportMaxPort {
    pub const DEFAULT_MIN_PORT: u16 = 5555;
    pub const DEFAULT_MAX_PORT: u16 = 5585;

    /// The maximum number of emulators that can be scanned. Default is 16.
    pub fn max_emulators() -> u16 {
        Self::get().unwrap_or(Self::DEFAULT_MAX_PORT) - Self::DEFAULT_MIN_PORT + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adb_local_transport_max_port() {
        AdbLocalTransportMaxPort::remove();
        assert!(AdbLocalTransportMaxPort::get().is_err());

        let ok = 5999;
        AdbLocalTransportMaxPort::set(ok);
        assert_eq!(AdbLocalTransportMaxPort::get().unwrap(), ok);
        assert_eq!(AdbLocalTransportMaxPort::max_emulators(), 445);

        let err = 5554;
        AdbLocalTransportMaxPort::set(err);
        assert_ne!(AdbLocalTransportMaxPort::get().unwrap(), err);
        assert_eq!(
            AdbLocalTransportMaxPort::get().unwrap(),
            AdbLocalTransportMaxPort::DEFAULT_MAX_PORT
        );

        AdbLocalTransportMaxPort::clear();
        assert_eq!(
            AdbLocalTransportMaxPort::get().unwrap(),
            AdbLocalTransportMaxPort::DEFAULT_MAX_PORT
        );
    }
}
