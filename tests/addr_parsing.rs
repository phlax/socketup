use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[test]
fn parses_numeric_targets() {
    let addr = socketup::addr::parse_target("127.0.0.1:8080").expect("must parse");
    assert_eq!(addr, SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080));

    let v6 = socketup::addr::parse_target("[::1]:8080").expect("must parse");
    assert_eq!(v6.port(), 8080);
}

#[test]
fn expands_shorthand_ports_to_loopback() {
    let bare = socketup::addr::parse_target("8080").expect("must parse");
    let with_colon = socketup::addr::parse_target(":8080").expect("must parse");
    let expected = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    assert_eq!(bare, expected);
    assert_eq!(with_colon, expected);
}

#[test]
fn rejects_hostnames_and_invalid_values() {
    assert!(socketup::addr::parse_target("localhost:8080").is_err());
    assert!(socketup::addr::parse_target("example.com:80").is_err());
    assert!(socketup::addr::parse_target("foo:1234").is_err());
    assert!(socketup::addr::parse_target("127.0.0.1:0").is_err());
}
