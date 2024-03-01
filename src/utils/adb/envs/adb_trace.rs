use super::AdbEnvVar;
use std::env::{self, VarError};
use std::fmt::Display;
use std::str::FromStr;

/// `$ADB_TRACE`: Comma (or space) separated list of debug info to log:
/// - [`AdbTrace::All`] "all"
/// - [`AdbTrace::Adb`] "adb"
/// - [`AdbTrace::Sockets`] "sockets"
/// - [`AdbTrace::Packets`] "packets"
/// - [`AdbTrace::Rwx`] "rwx"
/// - [`AdbTrace::Usb`] "usb"
/// - [`AdbTrace::Sync`] "sync"
/// - [`AdbTrace::Sysdeps`] "sysdeps"
/// - [`AdbTrace::Transport`] "transport"
/// - [`AdbTrace::Jdwp`] "jdwp"
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AdbTrace {
    All,
    Adb,
    Sockets,
    Packets,
    Rwx,
    Usb,
    Sync,
    Sysdeps,
    Transport,
    Jdwp,
}

impl AdbTrace {
    pub const ALL: &'static str = "all";
    pub const ADB: &'static str = "adb";
    pub const SOCKETS: &'static str = "sockets";
    pub const PACKETS: &'static str = "packets";
    pub const RWX: &'static str = "rwx";
    pub const USB: &'static str = "usb";
    pub const SYNC: &'static str = "sync";
    pub const SYSDEPS: &'static str = "sysdeps";
    pub const TRANSPORT: &'static str = "transport";
    pub const JDWP: &'static str = "jdwp";

    /// The string representation of the enum variant.
    pub const fn as_str(&self) -> &str {
        match self {
            Self::All => Self::ALL,
            Self::Adb => Self::ADB,
            Self::Sockets => Self::SOCKETS,
            Self::Packets => Self::PACKETS,
            Self::Rwx => Self::RWX,
            Self::Usb => Self::USB,
            Self::Sync => Self::SYNC,
            Self::Sysdeps => Self::SYSDEPS,
            Self::Transport => Self::TRANSPORT,
            Self::Jdwp => Self::JDWP,
        }
    }
}

impl AdbEnvVar for AdbTrace {
    type Value = Vec<Self>;

    const NAME: &'static str = "ADB_TRACE";

    fn get() -> Result<Self::Value, VarError> {
        Ok(env::var(Self::NAME)?
            .split([',', ' '])
            .filter_map(|r| r.parse().ok())
            .collect())
    }

    fn set<T: Into<Self::Value>>(var: T) {
        env::set_var(
            Self::NAME,
            var.into()
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
    }
}

impl AsRef<str> for AdbTrace {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for AdbTrace {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            AdbTrace::ALL => Ok(AdbTrace::All),
            AdbTrace::ADB => Ok(AdbTrace::Adb),
            AdbTrace::SOCKETS => Ok(AdbTrace::Sockets),
            AdbTrace::PACKETS => Ok(AdbTrace::Packets),
            AdbTrace::RWX => Ok(AdbTrace::Rwx),
            AdbTrace::USB => Ok(AdbTrace::Usb),
            AdbTrace::SYNC => Ok(AdbTrace::Sync),
            AdbTrace::SYSDEPS => Ok(AdbTrace::Sysdeps),
            AdbTrace::TRANSPORT => Ok(AdbTrace::Transport),
            AdbTrace::JDWP => Ok(AdbTrace::Jdwp),
            _ => Err(format!("Invalid value: {s}")),
        }
    }
}

impl Display for AdbTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::adb::envs::tests::test_env_var_helper;

    #[test]
    fn test_adb_trace() {
        let value = [
            AdbTrace::All,
            AdbTrace::Adb,
            AdbTrace::Sockets,
            AdbTrace::Packets,
            AdbTrace::Rwx,
            AdbTrace::Usb,
            AdbTrace::Sync,
            AdbTrace::Sysdeps,
            AdbTrace::Transport,
            AdbTrace::Jdwp,
        ];
        test_env_var_helper::<AdbTrace>(&[value[1..].to_vec(), value.to_vec()], &[], vec![]);
    }
}
