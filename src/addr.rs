use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

#[derive(Debug)]
pub enum AddrError {
    Empty,
    InvalidPort,
    PortZero,
    NonNumeric,
}

impl fmt::Display for AddrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddrError::Empty => write!(f, "empty target"),
            AddrError::InvalidPort => write!(f, "invalid port"),
            AddrError::PortZero => write!(f, "port must be > 0"),
            AddrError::NonNumeric => write!(f, "target must be numeric ip:port"),
        }
    }
}

pub fn parse_target(input: &str) -> Result<SocketAddr, AddrError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(AddrError::Empty);
    }

    if let Some(port_str) = input.strip_prefix(':') {
        return parse_loopback_shorthand(port_str);
    }

    if input.chars().all(|ch| ch.is_ascii_digit()) {
        return parse_loopback_shorthand(input);
    }

    let addr = SocketAddr::from_str(input).map_err(|_| AddrError::NonNumeric)?;
    if addr.port() == 0 {
        return Err(AddrError::PortZero);
    }
    Ok(addr)
}

fn parse_loopback_shorthand(port_str: &str) -> Result<SocketAddr, AddrError> {
    let port: u16 = port_str.parse().map_err(|_| AddrError::InvalidPort)?;
    if port == 0 {
        return Err(AddrError::PortZero);
    }

    Ok(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port,
    ))
}
