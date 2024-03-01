use super::AdbEnvVar;
use std::env::{self, VarError};

/// `$ANDROID_SERIAL`: Serial number to connect to (see -s).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AndroidSerial;

impl AdbEnvVar for AndroidSerial {
    type Value = String;

    const NAME: &'static str = "ANDROID_SERIAL";

    fn get() -> Result<Self::Value, VarError> {
        env::var(Self::NAME)
    }

    fn set<T: Into<Self::Value>>(var: T) {
        env::set_var(Self::NAME, var.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::adb::envs::tests::test_env_var_helper;

    #[test]
    fn test_android_serial() {
        test_env_var_helper::<AndroidSerial>(&["test_serial".to_string()], &[], String::new());
    }
}
