//! This module provides the [`AdbSocketFamily`] enum,
//! which represents the address family of the `adb` command.
//! It is used to specify the address of the `adb` server.

use std::ffi::c_uint;
use std::fmt::Display;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use crate::utils::adb::error::OptionParseError;

/// The address family of the `adb` command.
///
/// String can be converted into this enum using [`FromStr`] trait.
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
        let (family, value) = s.split_once(':').ok_or_else(|| {
            OptionParseError::with_reason(
                s.to_string(),
                stringify!(AdbSocketFamily),
                "unknown socket specification",
            )
        })?;
        match family {
            Self::TCP_STR => {
                if let Ok(socket_addr) = value.parse::<SocketAddr>() {
                    Ok(socket_addr.into())
                } else if let Ok(port) = value.parse::<u16>() {
                    Ok(AdbSocketFamily::Tcp {
                        host: None,
                        port: Some(port),
                    })
                } else if let Ok(ipv4) = value.parse::<Ipv4Addr>() {
                    Ok(AdbSocketFamily::Tcp {
                        host: Some(IpAddr::V4(ipv4)),
                        port: None,
                    })
                } else if value.starts_with('[') && value.ends_with(']') {
                    Ok(AdbSocketFamily::Tcp {
                        host: Some(IpAddr::V6(value[1..value.len() - 1].parse().map_err(
                            |e| {
                                OptionParseError::with_reason(
                                    s,
                                    stringify!(AdbSocketFamily::Tcp),
                                    format!("failed to parse `{value}` into `Ipv6Addr`: {e}"),
                                )
                            },
                        )?)),
                        port: None,
                    })
                } else {
                    Err(OptionParseError::with_reason(
                        s.to_string(),
                        stringify!(AdbSocketFamily::Tcp),
                        format!("failed to parse `{value}` into `SocketAddr`, `IpAddr` or `u16`"),
                    ))
                }
            }
            Self::LOCAL_ABSTRACT_STR => {
                if value.is_empty() {
                    Err(OptionParseError::with_reason(
                        s,
                        stringify!(AdbSocketFamily::LocalAbstract),
                        "empty socket name",
                    ))
                } else {
                    Ok(AdbSocketFamily::LocalAbstract(value.to_string()))
                }
            }
            Self::LOCAL_RESERVED_STR => {
                if value.is_empty() {
                    Err(OptionParseError::with_reason(
                        s,
                        stringify!(AdbSocketFamily::LocalReserved),
                        "empty socket name",
                    ))
                } else {
                    Ok(AdbSocketFamily::LocalReserved(value.to_string()))
                }
            }
            Self::LOCAL_FILE_SYSTEM_STR => {
                if value.is_empty() {
                    Err(OptionParseError::with_reason(
                        s,
                        stringify!(AdbSocketFamily::LocalFileSystem),
                        "empty socket name",
                    ))
                } else {
                    Ok(AdbSocketFamily::LocalFileSystem(PathBuf::from(value)))
                }
            }
            Self::DEV_STR => {
                if value.is_empty() {
                    Err(OptionParseError::with_reason(
                        s,
                        stringify!(AdbSocketFamily::Dev),
                        "empty character device name",
                    ))
                } else {
                    Ok(AdbSocketFamily::Dev(PathBuf::from(value)))
                }
            }
            Self::JDWP_STR => value.parse().map(AdbSocketFamily::Jdwp).map_err(|e| {
                OptionParseError::with_reason(
                    s,
                    stringify!(AdbSocketFamily::Jdwp),
                    format!("failed to parse `{value}` into `c_uint`: {e}"),
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
            Self::ACCEPT_FD_STR => Ok(AdbSocketFamily::AcceptFd(value.parse::<c_uint>().map_err(
                |e| OptionParseError::with_reason(s, stringify!(AdbSocketFamily::AcceptFd), e),
            )?)),
            _ => Err(OptionParseError::with_reason(
                s,
                stringify!(AdbSocketFamily),
                format!("unknown address family: `{}`", family),
            )),
        }
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

    #[test]
    fn test_adb_socket_family_tcp_parse() {
        let ok = [
            (
                "tcp:127.0.0.1:5555",
                AdbSocketFamily::Tcp {
                    host: Some(Ipv4Addr::new(127, 0, 0, 1).into()),
                    port: Some(5555),
                },
            ),
            (
                "tcp:[::1]:5555",
                AdbSocketFamily::Tcp {
                    host: Some(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).into()),
                    port: Some(5555),
                },
            ),
            (
                "tcp:127.0.0.1",
                AdbSocketFamily::Tcp {
                    host: Some(Ipv4Addr::new(127, 0, 0, 1).into()),
                    port: None,
                },
            ),
            (
                "tcp:[::1]",
                AdbSocketFamily::Tcp {
                    host: Some(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).into()),
                    port: None,
                },
            ),
            (
                "tcp:5555",
                AdbSocketFamily::Tcp {
                    host: None,
                    port: Some(5555),
                },
            ),
            (
                "tcp:0",
                AdbSocketFamily::Tcp {
                    host: None,
                    port: Some(0),
                },
            ),
        ];
        for (input, expected) in ok {
            assert_eq!(input.parse(), Ok(expected));
        }
        let err = [
            // incomplete address
            "tcp",
            "tcp:",
            "5555",
            "127.0.0.1",
            "127.0.0.1:5555",
            // Ipv6 address without square brackets
            "tcp:::",
            "tcp:::1",
            // IpAddr out of range
            "tcp:256.0.0.0",
            "tcp:256.-1.0.0",
            // port out of range
            "tcp:-1",
            "tcp:65536",
            // SocketAddr out of range
            "tcp:256.0.0.0:-1",
            "tcp:256.0.0.0:5555",
            "tcp:256.0.0.0:65536",
            "tcp:256.-1.0.0:5555",
            // invalid address
            "tcp:abcd",
            "tcp:a.b.c.d",
            "tcp:a.b.c.d:p",
        ];
        for input in err {
            assert!(input.parse::<AdbSocketFamily>().is_err());
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
