//! This module provides the [`AdbSocketFamily`] enum,
//! which represents the address family of the `adb` command.
//! It is used to specify the address of the `adb` server.

use std::ffi::c_uint;
use std::fmt::Display;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::ops::Index;
use std::path::PathBuf;
use std::str::FromStr;
use std::vec;

use crate::utils::adb::error::OptionParseError;

/// The address family of the `adb` command.
///
/// String can be converted into this enum using [`FromStr`] trait.
///
/// Note that the domain name (often "localhost") cannot be parsed directly into this enum.
/// Use [`AdbSocketFamily::from_resolved_str`] to resolve the domain name if needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdbSocketFamily {
    /// A TCP socket. Both Ipv4 and Ipv6 addresses are supported.
    ///
    /// A [`SocketAddr`] can be directly converted into this variant,
    /// [`IpAddr`] as well (`port` is [`None`]).
    ///
    /// # Equivalent form
    ///
    /// `tcp:[host:[port]]`
    ///
    /// ```
    /// use disanger::utils::adb::socket::AdbSocketFamily::Tcp;
    ///
    /// assert_eq!(
    ///     Tcp { host: Some("127.0.0.1".parse().unwrap()), port: Some(5555) }.to_string(),
    ///     "tcp:127.0.0.1:5555"
    /// );
    /// assert_eq!(
    ///     Tcp { host: Some("::1".parse().unwrap()), port: Some(5555) }.to_string(),
    ///     "tcp:[::1]:5555"
    /// );    
    /// assert_eq!(
    ///     Tcp { host: Some("127.0.0.1".parse().unwrap()), port: None }.to_string(),
    ///     "tcp:127.0.0.1"
    /// );
    /// assert_eq!(
    ///     Tcp { host: Some("::1".parse().unwrap()), port: None }.to_string(),
    ///     "tcp:[::1]"
    /// );
    /// assert_eq!(Tcp { host: None, port: Some(5555) }.to_string(), "tcp:5555");
    /// // Semantically invalid. Displayed as an empty string.
    /// assert_eq!(Tcp { host: None, port: None }.to_string(), "");
    /// ```
    ///
    /// # Note
    ///
    /// IPv6 address **must** be enclosed in square brackets. Without square brackets,
    /// it is impossible to distinguish between an IPv6 address and a port number.
    ///
    /// To avoid this issue, Ipv6 address without square brackets is simply not accepted.
    ///
    /// ```
    /// use disanger::utils::adb::socket::AdbSocketFamily;
    ///
    /// let wrong_addr = "tcp:::1:5555";  // "[::1]:5555" or "[::1:5555]:None"?
    /// assert!(wrong_addr.parse::<AdbSocketFamily>().is_err());
    /// ```
    Tcp {
        host: Option<IpAddr>,
        port: Option<u16>,
    },
    /// A Unix domain socket in the abstract namespace.
    ///
    /// # Equivalent form
    ///
    /// `localabstract:<unix domain socket name>`
    ///
    /// ```
    /// use disanger::utils::adb::socket::AdbSocketFamily::LocalAbstract;
    ///
    /// assert_eq!(
    ///     LocalAbstract("an_abstract_socket".to_string()).to_string(),
    ///     "localabstract:an_abstract_socket"
    /// );
    /// ```
    LocalAbstract(String),
    /// A Unix domain socket in the reserved namespace.
    ///
    /// # Equivalent form
    ///
    ///`localreserved:<unix domain socket name>`
    ///
    /// ```
    /// use disanger::utils::adb::socket::AdbSocketFamily::LocalReserved;
    ///
    /// assert_eq!(
    ///     LocalReserved("a_reserved_socket".to_string()).to_string(),
    ///     "localreserved:a_reserved_socket"
    /// );
    /// ```
    LocalReserved(String),
    /// A Unix domain socket in the file system.
    ///
    /// # Equivalent form
    ///
    /// `localfilesystem:<unix domain socket name>`
    ///
    /// ```
    /// use disanger::utils::adb::socket::AdbSocketFamily::LocalFileSystem;
    ///
    /// assert_eq!(
    ///     LocalFileSystem("/path/to/local_file_system_socket".into()).to_string(),
    ///     "localfilesystem:/path/to/local_file_system_socket"
    /// );
    /// ```
    LocalFileSystem(PathBuf),
    /// A character device.
    ///
    /// # Equivalent form
    ///
    /// `dev:<character device name>`
    ///
    /// ```
    /// use disanger::utils::adb::socket::AdbSocketFamily::Dev;
    ///
    /// assert_eq!(
    ///     Dev("/path/to/dev_socket".into()).to_string(),
    ///     "dev:/path/to/dev_socket"
    /// );
    /// ```
    Dev(PathBuf),
    /// A Java Debug Wire Protocol process.
    ///
    /// # Equivalent form
    ///
    /// `jdwp:<process pid>`
    ///
    /// ```
    /// use disanger::utils::adb::socket::AdbSocketFamily::Jdwp;
    ///
    /// assert_eq!(Jdwp(1234).to_string(), "jdwp:1234");
    /// ```
    Jdwp(c_uint),
    /// A VSOCK address.
    ///
    /// # Equivalent form
    ///
    /// `vsock:<CID>:<port>`
    ///
    /// ```
    /// use disanger::utils::adb::socket::AdbSocketFamily::Vsock;
    ///
    /// assert_eq!(Vsock { cid: 1, port: 2 }.to_string(), "vsock:1:2");
    Vsock { cid: c_uint, port: c_uint },
    /// A file descriptor for a socket.
    ///
    /// # Equivalent form
    ///
    /// `acceptfd:<fd>`
    ///
    /// ```
    /// use disanger::utils::adb::socket::AdbSocketFamily::AcceptFd;
    ///
    /// assert_eq!(AcceptFd(3).to_string(), "acceptfd:3");
    AcceptFd(c_uint),
}

impl AdbSocketFamily {
    // string representations of the address families

    const TCP_STR: &'static str = "tcp";
    const LOCAL_ABSTRACT_STR: &'static str = "localabstract";
    const LOCAL_RESERVED_STR: &'static str = "localreserved";
    const LOCAL_FILE_SYSTEM_STR: &'static str = "localfilesystem";
    const DEV_STR: &'static str = "dev";
    const JDWP_STR: &'static str = "jdwp";
    const VSOCK_STR: &'static str = "vsock";
    const ACCEPT_FD_STR: &'static str = "acceptfd";

    // consts for Vsock

    /// `VMADDR_CID_ANY` (-1U) means any address for binding.
    pub const VMADDR_CID_ANY: c_uint = c_uint::MAX;
    /// `VMADDR_CID_HYPERVISOR` (0) is reserved for services built into the hypervisor.
    pub const VMADDR_CID_HYPERVISOR: c_uint = 0;
    /// `VMADDR_CID_LOCAL` (1) is the well-known address for local communication (loopback).
    pub const VMADDR_CID_LOCAL: c_uint = 1;
    /// `VMADDR_CID_HOST` (2) is the well-known address of the host.
    pub const VMADDR_CID_HOST: c_uint = 2;
    /// `VMADDR_PORT_ANY` (-1U) means any port number for binding.
    pub const VMADDR_PORT_ANY: c_uint = c_uint::MAX;

    /// Parses a string slice into an `AdbSocketFamily`, resolve the domain name if needed.
    /// This only affects the [`AdbSocketFamily::Tcp`] variant. When converting to other variants,
    /// this function behaves the same as [`AdbSocketFamily::from_str`].
    ///
    /// The resolution may block the current thread while resolution is performed, which can be
    /// up to several seconds if the domain name is not resolvable. Prefer using [`str::parse`] or
    /// [`AdbSocketFamily::from_str`] to avoid this overhead.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr};
    /// use disanger::utils::adb::socket::AdbSocketFamily;
    ///
    /// assert_eq!(
    ///     AdbSocketFamily::from_resolved_str("tcp:localhost:5555"),
    ///     Ok(AdbSocketFamily::Tcp {
    ///         host: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
    ///         port: Some(5555)
    ///     })
    /// );
    pub fn from_resolved_str(s: &str) -> Result<Self, OptionParseError> {
        Self::from_str_helper(s, true)
    }

    fn resolve(
        source: &str,
        mut addrs: vec::IntoIter<SocketAddr>,
    ) -> Result<SocketAddr, OptionParseError> {
        addrs
            .clone()
            .find(|addr| addr.is_ipv4())
            .or_else(|| addrs.next())
            .map_or_else(
                || {
                    Err(OptionParseError::with_reason(
                        source,
                        stringify!(AdbSocketFamily::Tcp),
                        "empty iterator from `to_socket_addrs`",
                    ))
                },
                Ok,
            )
    }

    fn from_str_helper(s: &str, enable_resolve: bool) -> Result<Self, OptionParseError> {
        let (family, value) = s.split_once(':').ok_or_else(|| {
            OptionParseError::with_reason(
                s.to_string(),
                stringify!(AdbSocketFamily),
                "unknown socket specification",
            )
        })?;
        match family {
            Self::TCP_STR => Self::from_str_tcp_helper(s, value, enable_resolve),
            Self::LOCAL_ABSTRACT_STR => Self::from_str_str_value_helper(
                value,
                stringify!(AdbSocketFamily::LocalAbstract),
                AdbSocketFamily::LocalAbstract,
            ),
            Self::LOCAL_RESERVED_STR => Self::from_str_str_value_helper(
                value,
                stringify!(AdbSocketFamily::LocalReserved),
                AdbSocketFamily::LocalReserved,
            ),
            Self::LOCAL_FILE_SYSTEM_STR => Self::from_str_str_value_helper(
                value,
                stringify!(AdbSocketFamily::LocalFileSystem),
                |v| AdbSocketFamily::LocalFileSystem(PathBuf::from(v)),
            ),
            Self::DEV_STR => {
                Self::from_str_str_value_helper(value, stringify!(AdbSocketFamily::Dev), |v| {
                    AdbSocketFamily::Dev(PathBuf::from(v))
                })
            }
            Self::JDWP_STR => value.parse().map(AdbSocketFamily::Jdwp).map_err(|e| {
                OptionParseError::with_reason(
                    s,
                    stringify!(AdbSocketFamily::Jdwp),
                    format!("failed to parse `{value}` into unsigned int: {e}"),
                )
            }),
            Self::VSOCK_STR => match value.split_once(':').map(|(c, p)| (c.parse(), p.parse())) {
                Some((Ok(cid), Ok(port))) => Ok(AdbSocketFamily::Vsock { cid, port }),
                _ => Err(OptionParseError::with_reason(
                    s,
                    stringify!(AdbSocketFamily::Vsock),
                    format!("failed to parse CID and port from `{value}`"),
                )),
            },
            Self::ACCEPT_FD_STR => value.parse().map(AdbSocketFamily::AcceptFd).map_err(|e| {
                OptionParseError::with_reason(
                    s,
                    stringify!(AdbSocketFamily::AcceptFd),
                    format!("failed to parse `{value}` into unsigned int: {e}"),
                )
            }),
            _ => Err(OptionParseError::with_reason(
                s,
                stringify!(AdbSocketFamily),
                format!("unknown address family: `{}`", family),
            )),
        }
    }

    fn from_str_tcp_helper(
        source: &str,
        value: &str,
        enable_resolve: bool,
    ) -> Result<Self, OptionParseError> {
        if value.is_empty() {
            return Err(OptionParseError::with_reason(
                source,
                stringify!(AdbSocketFamily::Tcp),
                "empty address",
            ));
        }
        // `tcp:<port>`
        if let Ok(p) = value.parse::<i128>() {
            return if p >= 0 && p < u16::MAX as i128 {
                Ok(Self::Tcp {
                    host: None,
                    port: Some(p as u16),
                })
            } else {
                Err(OptionParseError::with_reason(
                    source,
                    stringify!(AdbSocketFamily::Tcp),
                    format!("port number `{value}` is out of range [0, 65535]"),
                ))
            };
        }
        // `tcp:<ipv4>:<port>` or `tcp:[<ipv6>]:<port>`
        if let Ok(socket_addr) = value.parse::<SocketAddr>() {
            return Ok(socket_addr.into());
        }
        // `tcp:<ipv4>`
        if let Ok(ipv4) = value.parse::<Ipv4Addr>() {
            return Ok(IpAddr::V4(ipv4).into());
        }
        // `tcp:[<ipv6>]`
        if value.starts_with('[') && value.ends_with(']') {
            return match value.index(1..value.len() - 1).parse::<Ipv6Addr>() {
                Ok(ipv6) => Ok(IpAddr::V6(ipv6).into()),
                Err(e) => Err(OptionParseError::with_reason(
                    source,
                    stringify!(AdbSocketFamily::Tcp),
                    format!("failed to parse `{value}` into `Ipv6Addr`: {e}"),
                )),
            };
        }
        if enable_resolve {
            // `tcp:<domain name>:<port>`
            if let Ok(addrs) = value.to_socket_addrs() {
                return Self::resolve(source, addrs).map(|addr| addr.into());
            }
            // `tcp:<domain name>`
            if let Ok(socket_addrs) = format!("{value}:0").to_socket_addrs() {
                return Self::resolve(source, socket_addrs).map(|addr| Self::Tcp {
                    host: Some(addr.ip()),
                    port: None,
                });
            }
        }
        Err(OptionParseError::with_reason(
            source,
            stringify!(AdbSocketFamily::Tcp),
            format!("`{value}` is not a valid address or port number"),
        ))
    }

    fn from_str_str_value_helper<F: FnOnce(String) -> Self>(
        value: &str,
        target: &str,
        f: F,
    ) -> Result<Self, OptionParseError> {
        if value.is_empty() {
            Err(OptionParseError::with_reason(
                value,
                target,
                "empty socket name",
            ))
        } else {
            Ok(f(value.to_string()))
        }
    }
}

impl From<IpAddr> for AdbSocketFamily {
    fn from(addr: IpAddr) -> Self {
        AdbSocketFamily::Tcp {
            host: Some(addr),
            port: None,
        }
    }
}

impl From<SocketAddr> for AdbSocketFamily {
    fn from(addr: SocketAddr) -> Self {
        AdbSocketFamily::Tcp {
            host: Some(addr.ip()),
            port: Some(addr.port()),
        }
    }
}

impl FromStr for AdbSocketFamily {
    type Err = OptionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AdbSocketFamily::from_str_helper(s, false)
    }
}

impl Display for AdbSocketFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdbSocketFamily::Tcp { host, port } => match (host, port) {
                (Some(IpAddr::V4(h)), Some(p)) => write!(f, "{}:{h}:{p}", Self::TCP_STR),
                (Some(IpAddr::V6(h)), Some(p)) => write!(f, "{}:[{h}]:{p}", Self::TCP_STR),
                (Some(IpAddr::V4(h)), None) => write!(f, "{}:{h}", Self::TCP_STR),
                (Some(IpAddr::V6(h)), None) => write!(f, "{}:[{h}]", Self::TCP_STR),
                (None, Some(p)) => write!(f, "{}:{p}", Self::TCP_STR),
                (None, None) => Ok(()),
            },
            AdbSocketFamily::LocalAbstract(path) => {
                write!(f, "{}:{}", Self::LOCAL_ABSTRACT_STR, path)
            }
            AdbSocketFamily::LocalReserved(path) => {
                write!(f, "{}:{}", Self::LOCAL_RESERVED_STR, path)
            }
            AdbSocketFamily::LocalFileSystem(path) => {
                write!(f, "{}:{}", Self::LOCAL_FILE_SYSTEM_STR, path.display())
            }
            AdbSocketFamily::Dev(path) => write!(f, "{}:{}", Self::DEV_STR, path.display()),
            AdbSocketFamily::Jdwp(pid) => write!(f, "{}:{}", Self::JDWP_STR, pid),
            AdbSocketFamily::Vsock { cid, port } => {
                write!(f, "{}:{}:{}", Self::VSOCK_STR, cid, port)
            }
            AdbSocketFamily::AcceptFd(fd) => write!(f, "{}:{}", Self::ACCEPT_FD_STR, fd),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

    #[test]
    fn test_adb_socket_family_display() {
        let data = [
            (
                AdbSocketFamily::Tcp {
                    host: None,
                    port: None,
                },
                "",
            ),
            (
                AdbSocketFamily::Tcp {
                    host: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                    port: None,
                },
                "tcp:127.0.0.1",
            ),
            (
                AdbSocketFamily::Tcp {
                    host: Some(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))),
                    port: None,
                },
                "tcp:[::1]",
            ),
            (
                AdbSocketFamily::Tcp {
                    host: None,
                    port: Some(5555),
                },
                "tcp:5555",
            ),
            (
                AdbSocketFamily::Tcp {
                    host: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                    port: Some(5555),
                },
                "tcp:127.0.0.1:5555",
            ),
            (
                AdbSocketFamily::Tcp {
                    host: Some(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))),
                    port: Some(5555),
                },
                "tcp:[::1]:5555",
            ),
            (
                AdbSocketFamily::LocalAbstract("local-abstract-socket".to_string()),
                "localabstract:local-abstract-socket",
            ),
            (
                AdbSocketFamily::LocalReserved("local-reserved-socket".to_string()),
                "localreserved:local-reserved-socket",
            ),
            (
                AdbSocketFamily::LocalFileSystem(PathBuf::from(
                    "/path/to/local-file-system-socket",
                )),
                "localfilesystem:/path/to/local-file-system-socket",
            ),
            (
                AdbSocketFamily::Dev(PathBuf::from("/path/to/dev-socket")),
                "dev:/path/to/dev-socket",
            ),
            (AdbSocketFamily::Jdwp(1234), "jdwp:1234"),
            (AdbSocketFamily::Vsock { cid: 1, port: 2 }, "vsock:1:2"),
            (AdbSocketFamily::AcceptFd(3), "acceptfd:3"),
        ];
        for (input, expected) in data {
            assert_eq!(input.to_string(), *expected);
        }
    }

    const TCP_PARSE_OK_NO_RESOLVE: [(&str, AdbSocketFamily); 6] = [
        (
            "127.0.0.1:5555",
            AdbSocketFamily::Tcp {
                host: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                port: Some(5555),
            },
        ),
        (
            "[::1]:5555",
            AdbSocketFamily::Tcp {
                host: Some(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))),
                port: Some(5555),
            },
        ),
        (
            "127.0.0.1",
            AdbSocketFamily::Tcp {
                host: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                port: None,
            },
        ),
        (
            "[::1]",
            AdbSocketFamily::Tcp {
                host: Some(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))),
                port: None,
            },
        ),
        (
            "5555",
            AdbSocketFamily::Tcp {
                host: None,
                port: Some(5555),
            },
        ),
        (
            "0",
            AdbSocketFamily::Tcp {
                host: None,
                port: Some(0),
            },
        ),
    ];

    const TCP_PARSE_ERR_NO_RESOLVE: [&str; 28] = [
        // incomplete address
        "127.0",
        "127.0:5555",
        "[]",
        "[]:5555",
        "[:]",
        "[:5555]",
        "5555:",
        // Ipv6 address without square brackets
        "::",
        "::1",
        "::1:5555",
        "ffff::1:5555",
        "1111:2222:3333:4444:5555:6666:7777:8888",
        "1111:2222:3333:4444:5555:6666:7777:8888:5555",
        // IpAddr out of range
        "256.0.0.0",
        "256.-1.0.0",
        "[gggg::]",
        "[::gggg]",
        // port out of range
        "-1",
        "65536",
        // SocketAddr out of range
        "256.0.0.0:-1",
        "256.0.0.0:5555",
        "256.0.0.0:65536",
        "256.-1.0.0:5555",
        "[gggg::]:5555",
        "[::gggg]:5555",
        // invalid characters
        "abcd",
        "a.b.c.d",
        "a.b.c.d:p",
    ];

    const TCP_PARSE_OK_RESOLVE: [(&str, AdbSocketFamily); 2] = [
        (
            "localhost:5555",
            AdbSocketFamily::Tcp {
                host: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                port: Some(5555),
            },
        ),
        (
            "localhost",
            AdbSocketFamily::Tcp {
                host: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                port: None,
            },
        ),
    ];

    const TCP_PARSE_ERR_RESOLVE: [&str; 6] = [
        "local-host",
        "local-host:5555",
        "localhost:",
        "abcd",
        "a.b.c.d",
        "a.b.c.d:p",
    ];

    #[test]
    fn test_adb_socket_family_tcp_from_str_helper_disable_resolve() {
        for (input, expected) in TCP_PARSE_OK_NO_RESOLVE {
            assert_eq!(
                AdbSocketFamily::from_str_tcp_helper("", input, false),
                Ok(expected)
            );
        }
        for input in TCP_PARSE_ERR_NO_RESOLVE {
            assert!(
                AdbSocketFamily::from_str_tcp_helper("", input, false).is_err(),
                "input:`{input}`"
            );
        }
    }

    #[test]
    fn test_adb_socket_family_tcp_from_str_helper_enable_resolve() {
        for (input, expected) in TCP_PARSE_OK_RESOLVE {
            assert_eq!(
                AdbSocketFamily::from_str_tcp_helper("", input, true),
                Ok(expected)
            );
        }
        // This loop is time-consuming. Because it tries to resolve the wrong domain name.
        for input in TCP_PARSE_ERR_RESOLVE {
            assert!(
                AdbSocketFamily::from_str_tcp_helper("", input, true).is_err(),
                "input:`{input}`"
            );
        }
    }

    #[test]
    fn test_adb_socket_family_tcp_parse() {
        for (input, expected) in TCP_PARSE_OK_NO_RESOLVE.map(|(i, e)| (format!("tcp:{i}"), e)) {
            assert_eq!(input.parse(), Ok(expected));
        }
        for input in TCP_PARSE_ERR_NO_RESOLVE.map(|i| format!("tcp:{i}")) {
            assert!(input.parse::<AdbSocketFamily>().is_err(), "input:`{input}`");
        }
        let extra_err = [
            "tcp",
            "tcp:",
            "5555",
            "127.0.0.1",
            "127.0.0.1:5555",
            "[::1]",
            "[::1]:5555",
            "::1",
        ];
        for input in extra_err {
            assert!(input.parse::<AdbSocketFamily>().is_err(), "input:`{input}`");
        }
    }

    #[test]
    fn test_adb_socket_family_with_str_value_parse() {
        let ok = [
            (
                "localabstract:local-abstract-socket",
                AdbSocketFamily::LocalAbstract("local-abstract-socket".to_string()),
            ),
            (
                "localreserved:local-reserved-socket",
                AdbSocketFamily::LocalReserved("local-reserved-socket".to_string()),
            ),
            (
                "localfilesystem:/path/to/local-file-system-socket",
                AdbSocketFamily::LocalFileSystem(PathBuf::from(
                    "/path/to/local-file-system-socket",
                )),
            ),
            (
                "dev:/path/to/dev-socket",
                AdbSocketFamily::Dev(PathBuf::from("/path/to/dev-socket")),
            ),
        ];
        for (input, expected) in ok {
            assert_eq!(input.parse(), Ok(expected));
        }
        let err = [
            "localabstract",
            "localabstract:",
            "localreserved",
            "localreserved:",
            "localfilesystem",
            "localfilesystem:",
            "dev",
            "dev:",
        ];
        for input in err {
            assert!(input.parse::<AdbSocketFamily>().is_err());
        }
    }

    #[test]
    fn test_adb_socket_family_with_uint_value_parse() {
        let ok = [
            ("jdwp:1234", AdbSocketFamily::Jdwp(1234)),
            ("vsock:1:2", AdbSocketFamily::Vsock { cid: 1, port: 2 }),
            ("acceptfd:3", AdbSocketFamily::AcceptFd(3)),
        ];
        for (input, expected) in ok {
            assert_eq!(input.parse(), Ok(expected));
        }
        let overflow = c_uint::MAX as u128 + 1;
        let err = [
            "jdwp",
            "jdwp:",
            "jdwp:-1",
            &format!("jdwp:{}", overflow),
            "vsock",
            "vsock:",
            "vsock:1",
            "vsock::1",
            "vsock:1:",
            "vsock:-1",
            "vsock:-1:-1",
            &format!("vsock:1:{}", overflow),
            &format!("vsock:{}:2", overflow),
            &format!("vsock:{}:{}", overflow, overflow),
            "acceptfd",
            "acceptfd:",
            "acceptfd:-1",
            &format!("acceptfd:{}", overflow),
        ];
        for input in err {
            assert!(input.parse::<AdbSocketFamily>().is_err());
        }
    }
}
