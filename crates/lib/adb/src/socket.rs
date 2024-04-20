//! This module provides some types representing the adb socket families.

use std::fmt::{Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
use std::path::PathBuf;
use std::str::FromStr;

use derive::AdbSocketFamily;

use crate::error::AdbError;

/// A marker trait for adb socket families.
///
/// By implementing this trait, a type guarantees that:
///
/// - It can be parsed from a valid adb socket family string.
/// - It can be displayed as a valid argument for an adb command.
pub trait AdbSocketFamily: FromStr + Display {}

/// The address families of the `adb` command.
#[derive(AdbSocketFamily, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AdbSocketFamilies {
    Tcp(Tcp),
    LocalAbstract(LocalAbstract),
    LocalReserved(LocalReserved),
    LocalFileSystem(LocalFileSystem),
    Dev(Dev),
    DevRaw(DevRaw),
    Jdwp(Jdwp),
    Vsock(Vsock),
    AcceptFd(AcceptFd),
}

/// A TCP socket. Both IPv4 and IPv6 addresses are supported.
///
/// # Syntax
///
/// `tcp:[host:[port]]`
///
/// - `host`: Optional hostname or IP address.
///     If an IPv6 address is provided, it should be enclosed in square brackets.
/// - `port`: Optional port number.
///
/// # Note
///
/// Semantically, `host` and `port` should not be None at the same time.
///
/// In this case, the `Tcp` socket is considered invalid and behaves as follows:
/// - The [`Display`] implementation will return an empty string.
/// - The [`FromStr`] implementation will return an error.
///
/// ```
/// # use adb::socket::Tcp;
/// assert!("tcp:".parse::<Tcp>().is_err());
/// assert_eq!(Tcp { ip: None, port: None }.to_string(), "");
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Tcp {
    // The IP address of the host.
    pub ip: Option<IpAddr>,
    // The port number.
    pub port: Option<u16>,
}

impl Tcp {
    /// Creates a new `Tcp` socket with the given IP address and port number.
    pub const fn new(host: IpAddr, port: u16) -> Self {
        Self {
            ip: Some(host),
            port: Some(port),
        }
    }

    /// Creates a new `Tcp` socket with the given IP address.
    pub const fn from_ip(host: IpAddr) -> Self {
        Self {
            ip: Some(host),
            port: None,
        }
    }

    /// Creates a new `Tcp` socket with the given IPv4 address.
    pub const fn from_ipv4(host: Ipv4Addr) -> Self {
        Self {
            ip: Some(IpAddr::V4(host)),
            port: None,
        }
    }

    /// Creates a new `Tcp` socket with the given IPv6 address.
    pub const fn from_ipv6(host: Ipv6Addr) -> Self {
        Self {
            ip: Some(IpAddr::V6(host)),
            port: None,
        }
    }

    /// Creates a new `Tcp` socket with the given port number.
    pub const fn from_port(port: u16) -> Self {
        Self {
            ip: None,
            port: Some(port),
        }
    }

    /// Resolves the given hostname into an IP address. If the resolution results
    /// in multiple IP addresses, IPv4 addresses are preferred.
    ///
    /// # Note
    ///
    /// The resolution may block the current thread while resolution is performed.
    /// If this is not desired, consider using [`FromStr`] which is non-blocking.
    ///
    /// # Examples
    ///
    /// ```
    /// use adb::socket::Tcp;
    /// use std::net::Ipv4Addr;
    ///
    /// let tcp = Tcp::from_host("localhost").unwrap();
    /// assert_eq!(tcp, Tcp::from_ipv4(Ipv4Addr::new(127, 0, 0, 1)));
    /// ```
    pub fn from_host(host: &str) -> Result<Self, AdbError> {
        host.parse().or_else(|_| {
            Self::resolve(host).or_else(|e| {
                // ToSocketAddrs requires a hostname with a port number.
                // Retry if the input hostname does not contain a port number,
                match Self::resolve(&format!("{host}:0")) {
                    Ok(tcp) => Ok(Self {
                        ip: tcp.ip,
                        port: None,
                    }),
                    _ => Err(e),
                }
            })
        })
    }

    fn resolve(host: &str) -> Result<Self, AdbError> {
        let mut addrs = host.to_socket_addrs().map_err(|e| AdbError::Parse {
            value: host.to_string(),
            source_type: "&str",
            target_type: "std::vec::IntoIter<SocketAddr>",
            source: Some(Box::new(e)),
        })?;
        let first = addrs.next();
        match first {
            None => Err(AdbError::Parse {
                value: host.to_string(),
                source_type: "&str",
                target_type: "SocketAddr",
                source: None,
            }),
            Some(SocketAddr::V4(v4)) => Ok(v4.into()),
            _ => Ok(addrs.find(SocketAddr::is_ipv4).or(first).unwrap().into()),
        }
    }
}

impl Display for Tcp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.ip, self.port) {
            (Some(IpAddr::V4(v4)), Some(port)) => write!(f, "tcp:{}:{}", v4, port),
            (Some(IpAddr::V6(v6)), Some(port)) => write!(f, "tcp:[{}]:{}", v6, port),
            (Some(IpAddr::V4(v4)), None) => write!(f, "tcp:{}", v4),
            (Some(IpAddr::V6(v6)), None) => write!(f, "tcp:[{}]", v6),
            (None, Some(port)) => write!(f, "tcp:{}", port),
            (None, None) => write!(f, ""),
        }
    }
}

impl FromStr for Tcp {
    type Err = AdbError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix("tcp:") {
            None | Some("") => Err(AdbError::Parse {
                value: s.to_string(),
                source_type: "&str",
                target_type: "Tcp",
                source: None,
            }),
            Some(value) => {
                if let Ok(port) = value.parse::<u16>() {
                    Ok(port.into())
                } else if let Ok(addr) = value.parse::<SocketAddr>() {
                    Ok(addr.into())
                } else if let Ok(v4) = value.parse::<Ipv4Addr>() {
                    Ok(v4.into())
                } else {
                    value
                        .strip_prefix('[')
                        .and_then(|value| value.strip_suffix(']'))
                        .map_or_else(
                            || {
                                Err(AdbError::Parse {
                                    value: value.to_string(),
                                    source_type: "&str",
                                    target_type: "Ipv6Addr",
                                    source: None,
                                })
                            },
                            |value| {
                                value.parse::<Ipv6Addr>().map_or_else(
                                    |e| {
                                        Err(AdbError::Parse {
                                            value: value.to_string(),
                                            source_type: "&str",
                                            target_type: "Ipv6Addr",
                                            source: Some(Box::new(e)),
                                        })
                                    },
                                    |v6| Ok(v6.into()),
                                )
                            },
                        )
                }
            }
        }
    }
}

impl AdbSocketFamily for Tcp {}

impl From<SocketAddr> for Tcp {
    fn from(addr: SocketAddr) -> Self {
        Self::new(addr.ip(), addr.port())
    }
}

impl From<SocketAddrV4> for Tcp {
    fn from(addr: SocketAddrV4) -> Self {
        Self::new(IpAddr::V4(*addr.ip()), addr.port())
    }
}

impl From<SocketAddrV6> for Tcp {
    fn from(addr: SocketAddrV6) -> Self {
        Self::new(IpAddr::V6(*addr.ip()), addr.port())
    }
}

impl From<IpAddr> for Tcp {
    fn from(ip: IpAddr) -> Self {
        Self::from_ip(ip)
    }
}

impl From<Ipv4Addr> for Tcp {
    fn from(ipv4: Ipv4Addr) -> Self {
        Self::from_ipv4(ipv4)
    }
}

impl From<Ipv6Addr> for Tcp {
    fn from(ipv6: Ipv6Addr) -> Self {
        Self::from_ipv6(ipv6)
    }
}

impl From<u16> for Tcp {
    fn from(port: u16) -> Self {
        Self::from_port(port)
    }
}

/// A Unix domain socket in the abstract namespace.
///
/// # Syntax
///
/// `localabstract:<unix domain socket name>`
#[derive(AdbSocketFamily, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct LocalAbstract(pub String);

/// A Unix domain socket in the reserved namespace.
///
/// # Syntax
///
///`localreserved:<unix domain socket name>`
#[derive(AdbSocketFamily, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct LocalReserved(pub String);

/// A Unix domain socket in the file system.
///
/// # Syntax
///
/// `localfilesystem:<unix domain socket name>`
#[derive(AdbSocketFamily, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct LocalFileSystem(pub PathBuf);

/// A character device.
///
/// # Syntax
///
/// `dev:<character device name>`
#[derive(AdbSocketFamily, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Dev(pub PathBuf);

/// Open device in raw mode.
///
/// # Syntax
///
/// `dev-raw:<character device name>`
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct DevRaw(pub PathBuf);

impl Display for DevRaw {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "dev-raw:{}", self.0.display())
    }
}

impl FromStr for DevRaw {
    type Err = AdbError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("dev-raw:").ok_or_else(|| AdbError::Parse {
            value: s.to_string(),
            source_type: "&str",
            target_type: "DevRaw",
            source: None,
        })?;
        Ok(Self(PathBuf::from(s)))
    }
}

impl AdbSocketFamily for DevRaw {}

/// A Java Debug Wire Protocol process.
///
/// # Syntax
///
/// `jdwp:<process pid>`
#[derive(AdbSocketFamily, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Jdwp(pub u32);

/// A VSOCK address.
///
/// # Syntax
///
/// `vsock:<cid>:<port>`
#[derive(AdbSocketFamily, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Vsock {
    pub cid: u32,
    pub port: u32,
}

/// A file descriptor for a socket.
///
/// # Syntax
///
/// `acceptfd:<fd>`
#[derive(AdbSocketFamily, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AcceptFd(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    const TCP_COMMON: [(&str, Tcp); 5] = [
        ("tcp:5555", Tcp::from_port(5555)),
        ("tcp:127.0.0.1", Tcp::from_ipv4(Ipv4Addr::new(127, 0, 0, 1))),
        (
            "tcp:[::1]",
            Tcp::from_ipv6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
        ),
        (
            "tcp:127.0.0.1:5555",
            Tcp::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5555),
        ),
        (
            "tcp:[::1]:5555",
            Tcp::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 5555),
        ),
    ];

    const TCP_PARSE_ERR: [&str; 30] = [
        "",
        "tcp:",
        // incomplete address
        "tcp:127.0",
        "tcp:127.0:5555",
        "tcp:[]",
        "tcp:[]:5555",
        "tcp:[:]",
        "tcp:[:5555]",
        "tcp:5555:",
        // Ipv6 address without square brackets
        "tcp:::",
        "tcp:::1",
        "tcp:::1:5555",
        "tcp:ffff::1:5555",
        "tcp:1111:2222:3333:4444:5555:6666:7777:8888",
        "tcp:1111:2222:3333:4444:5555:6666:7777:8888:5555",
        // IpAddr out of range
        "tcp:256.0.0.0",
        "tcp:256.-1.0.0",
        "tcp:[gggg::]",
        "tcp:[::gggg]",
        // port out of range
        "tcp:-1",
        "tcp:65536",
        // SocketAddr out of range
        "tcp:256.0.0.0:-1",
        "tcp:256.0.0.0:5555",
        "tcp:256.0.0.0:65536",
        "tcp:256.-1.0.0:5555",
        "tcp:[gggg::]:5555",
        "tcp:[::gggg]:5555",
        // invalid characters
        "tcp:abcd",
        "tcp:a.b.c.d",
        "tcp:a.b.c.d:p",
    ];

    #[test]
    fn test_tcp_display() {
        for (s, tcp) in TCP_COMMON {
            assert_eq!(s, tcp.to_string());
        }
    }

    #[test]
    fn test_tcp_parse() {
        for (s, tcp) in TCP_COMMON {
            assert_eq!(tcp, s.parse().unwrap());
        }
        for s in TCP_PARSE_ERR {
            assert!(s.parse::<Tcp>().is_err(), "{}", s);
        }
    }

    const TCP_RESOLVE_OK: [(&str, Tcp); 2] = [
        (
            "localhost:5555",
            Tcp::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5555),
        ),
        ("localhost", Tcp::from_ipv4(Ipv4Addr::new(127, 0, 0, 1))),
    ];

    const TCP_RESOLVE_ERR: [&str; 6] = [
        "local-host",
        "local-host:5555",
        "localhost:",
        "abcd",
        "a.b.c.d",
        "a.b.c.d:p",
    ];

    #[test]
    fn test_tcp_resolve() {
        for (s, tcp) in TCP_RESOLVE_OK {
            assert_eq!(tcp, Tcp::from_host(s).unwrap());
        }
        for s in TCP_RESOLVE_ERR {
            assert!(Tcp::from_host(s).is_err(), "{}", s);
        }
    }

    #[test]
    fn test_local_abstract_display() {
        let local_abstract = LocalAbstract("socket".to_string());
        assert_eq!("localabstract:socket", local_abstract.to_string());
    }

    #[test]
    fn test_local_abstract_parse() {
        let local_abstract = LocalAbstract("socket".to_string());
        assert_eq!(local_abstract, "localabstract:socket".parse().unwrap());
    }

    #[test]
    fn test_local_reserved_display() {
        let local_reserved = LocalReserved("socket".to_string());
        assert_eq!("localreserved:socket", local_reserved.to_string());
    }

    #[test]
    fn test_local_reserved_parse() {
        let local_reserved = LocalReserved("socket".to_string());
        assert_eq!(local_reserved, "localreserved:socket".parse().unwrap());
    }

    #[test]
    fn test_local_file_system_display() {
        let local_file_system = LocalFileSystem(PathBuf::from("/path/to/socket"));
        assert_eq!(
            "localfilesystem:/path/to/socket",
            local_file_system.to_string()
        );
    }

    #[test]
    fn test_local_file_system_parse() {
        let local_file_system = LocalFileSystem(PathBuf::from("/path/to/socket"));
        assert_eq!(
            local_file_system,
            "localfilesystem:/path/to/socket".parse().unwrap()
        );
    }

    #[test]
    fn test_dev_display() {
        let dev = Dev(PathBuf::from("/dev/tty"));
        assert_eq!("dev:/dev/tty", dev.to_string());
    }

    #[test]
    fn test_dev_parse() {
        let dev = Dev(PathBuf::from("/dev/tty"));
        assert_eq!(dev, "dev:/dev/tty".parse().unwrap());
    }

    #[test]
    fn test_dev_raw_display() {
        let dev_raw = DevRaw(PathBuf::from("/dev/tty"));
        assert_eq!("dev-raw:/dev/tty", dev_raw.to_string());
    }

    #[test]
    fn test_dev_raw_parse() {
        let dev_raw = DevRaw(PathBuf::from("/dev/tty"));
        assert_eq!(dev_raw, "dev-raw:/dev/tty".parse().unwrap());
    }

    const OVERFLOW: u64 = u32::MAX as u64 + 1;

    #[test]
    fn test_jdwp_display() {
        let jdwp = Jdwp(1234);
        assert_eq!("jdwp:1234", jdwp.to_string());
    }

    #[test]
    fn test_jdwp_parse() {
        let jdwp = Jdwp(1234);
        assert_eq!(jdwp, "jdwp:1234".parse().unwrap());
        let err = ["jdwp", "jdwp:", "jdwp:-1", &format!("jdwp:{}", OVERFLOW)];
        for s in &err {
            assert!(s.parse::<Jdwp>().is_err(), "{}", s);
        }
    }

    #[test]
    fn test_vsock_display() {
        let vsock = Vsock { cid: 1, port: 2 };
        assert_eq!("vsock:1:2", vsock.to_string());
    }

    #[test]
    fn test_vsock_parse() {
        let vsock = Vsock { cid: 1, port: 2 };
        assert_eq!(vsock, "vsock:1:2".parse().unwrap());
        let err = [
            "vsock",
            "vsock:",
            "vsock:1",
            "vsock::1",
            "vsock:1:",
            "vsock:-1",
            "vsock:-1:-1",
            &format!("vsock:1:{}", OVERFLOW),
            &format!("vsock:{}:2", OVERFLOW),
            &format!("vsock:{}:{}", OVERFLOW, OVERFLOW),
        ];
        for s in &err {
            assert!(s.parse::<Vsock>().is_err(), "{}", s);
        }
    }

    #[test]
    fn test_accept_fd_display() {
        let accept_fd = AcceptFd(1);
        assert_eq!("acceptfd:1", accept_fd.to_string());
    }

    #[test]
    fn test_accept_fd_parse() {
        let accept_fd = AcceptFd(1);
        assert_eq!(accept_fd, "acceptfd:1".parse().unwrap());
        let err = [
            "acceptfd",
            "acceptfd:",
            "acceptfd:-1",
            &format!("acceptfd:{}", OVERFLOW),
        ];
        for s in &err {
            assert!(s.parse::<AcceptFd>().is_err(), "{}", s);
        }
    }
}
