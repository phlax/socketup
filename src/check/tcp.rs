use super::ProbeResult;
use std::io;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub fn probe_tcp(target: SocketAddr, timeout: Duration) -> ProbeResult {
    match TcpStream::connect_timeout(&target, timeout) {
        Ok(_stream) => ProbeResult::Ok,
        Err(err) if err.kind() == io::ErrorKind::TimedOut => ProbeResult::Timeout,
        Err(err) if err.kind() == io::ErrorKind::ConnectionRefused => ProbeResult::ConnectRefused,
        Err(err) => ProbeResult::OtherErr(err),
    }
}
