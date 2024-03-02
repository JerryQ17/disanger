//! The global options of the `adb` command.

use crate::adb::error::OptionParseError;
use crate::adb::socket::AdbSocketFamily;
use std::fmt::Display;
use std::net::IpAddr;
use std::str::FromStr;

/// The global options of the `adb` command.
///
/// - [`GlobalOption::ListenAll`] `-a` Listen on all network interfaces, not just localhost.
/// - [`GlobalOption::Usb`] `-d` Use USB device (error if multiple devices connected).
/// - [`GlobalOption::Tcp`] `-e` Use TCP/IP device (error if multiple TCP/IP devices available).
/// - [`GlobalOption::Serial`] `-s SERIAL` Use device with given SERIAL (overrides $ANDROID_SERIAL).
/// - [`GlobalOption::OneDevice`] `-t ID` Use device with given transport ID.
/// - [`GlobalOption::Host`] `-H` Name of adb server host (default=`localhost`).
/// - [`GlobalOption::Port`] `-P *PORT` Smart socket PORT of adb server (default=`5037`).
/// - [`GlobalOption::Listen`] `-L SOCKET` Listen on given socket for adb server (default=`tcp:localhost:5037`).
/// - [`GlobalOption::OneDevice`] `--one-device SERIAL|USB` Server will only connect to one USB device,
/// specified by a SERIAL number or USB device address (only with ‘start-server’ or ‘server nodaemon’).
/// - [`GlobalOption::ExitOnWriteError`] `--exit-on-write-error` Exit if stdout is closed.
///
/// Copied from [GLOBAL OPTIONS](https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/master/docs/user/adb.1.md#global-options)
///
/// # Examples
///
/// ```
/// use disanger::adb::global_options::GlobalOption;
/// use disanger::adb::socket::AdbSocketFamily::Tcp;
/// use std::net::{IpAddr, Ipv4Addr};
///
/// assert_eq!("-a".parse(), Ok(GlobalOption::ListenAll));
/// assert_eq!(
///     "-s emulator-123".parse(),
///     Ok(GlobalOption::Serial("emulator-123".to_string()))
/// );
/// assert_eq!(
///     "-L tcp:127.0.0.1:8080".parse(),
///     Ok(GlobalOption::Listen(Tcp{
///         host: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
///         port: Some(8080)
///     }))
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalOption {
    /// `-a`: Listen on all network interfaces, not just localhost.
    ListenAll,
    /// `-d`: Use USB device (error if multiple devices connected).
    Usb,
    /// `-e`: Use TCP/IP device (error if multiple TCP/IP devices available).
    Tcp,
    /// `-s SERIAL`: Use device with given SERIAL (overrides $ANDROID_SERIAL).
    Serial(String),
    /// `-t ID`: Use device with given transport id.
    TransportId(String),
    /// `-H`: Name of adb server host. Default is `localhost`.
    Host(IpAddr),
    /// `-P *PORT`: Smart socket PORT of adb server. Default is `5037`.
    Port(u16),
    /// `-L SOCKET`: Listen on given socket for adb server. Default is `tcp:localhost:5037`.
    Listen(AdbSocketFamily),
    /// `--one-device SERIAL|USB`:
    /// Server will only connect to one USB device,
    /// specified by a SERIAL number or USB device address
    /// (only with `start-server` or `server nodaemon`).
    OneDevice(String),
    /// `--exit-on-write-error`: Exit if stdout is closed.
    ExitOnWriteError,
}

impl GlobalOption {
    /// Parse a string slice into a [`GlobalOption`], resolve the domain name if needed.
    /// This only affects the [`GlobalOption::Listen`] variant. When converting to other variants,
    /// this function behaves the same as [`GlobalOption::from_str`].
    ///
    /// The resolution may block the current thread while resolution is performed, which can be
    /// up to several seconds if the domain name is not resolvable. Prefer using [`str::parse`] or
    /// [`GlobalOption::from_str`] to avoid this overhead.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr};
    /// use disanger::adb::global_options::GlobalOption;
    /// use disanger::adb::socket::AdbSocketFamily::Tcp;
    ///
    /// assert_eq!(
    ///     GlobalOption::from_resolved_str("-L tcp:localhost:8080"),
    ///     Ok(GlobalOption::Listen(Tcp{
    ///         host: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
    ///         port: Some(8080)
    ///     }))
    /// );
    /// ```
    pub fn from_resolved_str(s: &str) -> Result<Self, OptionParseError> {
        Self::from_str_helper(s, true)
    }

    fn from_str_helper(s: &str, enable_resolve: bool) -> Result<Self, OptionParseError> {
        let trimmed = s.trim();
        // 1. Options that don't require a value.
        match trimmed {
            "-a" => return Ok(Self::ListenAll),
            "-d" => return Ok(Self::Usb),
            "-e" => return Ok(Self::Tcp),
            "--exit-on-write-error" => return Ok(Self::ExitOnWriteError),
            _ => {}
        };
        // 2. Split the value into the option and its value.
        let (opt, val) = trimmed.split_once(char::is_whitespace).ok_or_else(|| {
            OptionParseError::with_reason(s, stringify!(GlobalOption), "unknown global option")
        })?;
        // 3. Options that require a value.
        match opt {
            "-s" => Ok(Self::Serial(val.to_string())),
            "-t" => Ok(Self::TransportId(val.to_string())),
            "-H" => val
                .parse()
                .map(Self::Host)
                .map_err(|e| OptionParseError::with_reason(val, stringify!(IpAddr), e)),
            "-P" => val
                .parse()
                .map(Self::Port)
                .map_err(|e| e.to_string().into()),
            "-L" => {
                if enable_resolve {
                    Ok(Self::Listen(AdbSocketFamily::from_resolved_str(val)?))
                } else {
                    val.parse().map(Self::Listen)
                }
            }
            "--one-device" => Ok(Self::OneDevice(val.to_string())),
            _ => Err(format!("Invalid global option: {}", s).into()),
        }
    }
}

impl FromStr for GlobalOption {
    type Err = OptionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_helper(s, false)
    }
}

impl Display for GlobalOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ListenAll => write!(f, "-a"),
            Self::Usb => write!(f, "-d"),
            Self::Tcp => write!(f, "-e"),
            Self::Serial(serial) => write!(f, "-s {}", serial),
            Self::TransportId(id) => write!(f, "-t {}", id),
            Self::Host(ip) => write!(f, "-H {}", ip),
            Self::Port(port) => write!(f, "-P {}", port),
            Self::Listen(addr) => write!(f, "-L {}", addr),
            Self::OneDevice(device) => write!(f, "--one-device {}", device),
            Self::ExitOnWriteError => write!(f, "--exit-on-write-error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_loop<T: AsRef<str>>(arr: &[(T, GlobalOption)]) {
        for (s, expected) in arr {
            assert_eq!(s.as_ref().parse(), Ok(expected.clone()));
            assert_eq!(
                format!("\r\n\t {} \t\r\n", s.as_ref()).parse(),
                Ok(expected.clone())
            );
            assert!(GlobalOption::from_str(&format!("-{}", s.as_ref())).is_err());
        }
    }

    #[test]
    fn test_from_str_with_no_value() {
        test_loop(&[
            ("-a", GlobalOption::ListenAll),
            ("-d", GlobalOption::Usb),
            ("-e", GlobalOption::Tcp),
            ("--exit-on-write-error", GlobalOption::ExitOnWriteError),
        ]);
    }

    #[test]
    fn test_from_str_with_string_value() {
        let values = ["123", "test", "emulator-123", "127.0.0.1:1234"];
        let types = [
            ("-s", GlobalOption::Serial as fn(String) -> GlobalOption),
            (
                "-t",
                GlobalOption::TransportId as fn(String) -> GlobalOption,
            ),
            (
                "--one-device",
                GlobalOption::OneDevice as fn(String) -> GlobalOption,
            ),
        ];
        for (opt, f) in types {
            test_loop(
                &values
                    .iter()
                    .map(|s| (format!("{} {}", opt, s), f(s.to_string())))
                    .collect::<Vec<_>>(),
            );
        }
    }
}
