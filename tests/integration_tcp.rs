use std::net::TcpListener;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn write_config(config: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "socketup-integration-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("socketup.yaml");
    std::fs::write(&path, config).expect("write config");
    path
}

fn run_check(config: &std::path::Path, check_name: &str) -> i32 {
    let status = Command::new(env!("CARGO_BIN_EXE_socketup"))
        .arg("--config")
        .arg(config)
        .arg("check")
        .arg(check_name)
        .status()
        .expect("run check");

    status.code().unwrap_or(1)
}

#[test]
fn tcp_check_exit_codes_cover_success_failure_and_missing_check() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("addr").port();

    let config = format!(
        "checks:\n  admin:\n    type: tcp\n    targets:\n      - \"127.0.0.1:{}\"\n  closed:\n    type: tcp\n    targets:\n      - \"127.0.0.1:{}\"\n",
        port,
        port + 1
    );

    let path = write_config(&config);

    assert_eq!(run_check(&path, "admin"), 0);

    drop(listener);
    thread::sleep(Duration::from_millis(30));

    assert_eq!(run_check(&path, "admin"), 1);
    assert_eq!(run_check(&path, "missing"), 6);

    let _ = std::fs::remove_file(&path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}

#[test]
fn mode_any_succeeds_when_one_target_is_up() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let open_port = listener.local_addr().expect("addr").port();
    let closed_port = TcpListener::bind("127.0.0.1:0")
        .expect("port reservation")
        .local_addr()
        .expect("addr")
        .port();

    let config = format!(
        "checks:\n  all-check:\n    type: tcp\n    mode: all\n    targets:\n      - \"127.0.0.1:{open_port}\"\n      - \"127.0.0.1:{closed_port}\"\n  any-check:\n    type: tcp\n    mode: any\n    targets:\n      - \"127.0.0.1:{open_port}\"\n      - \"127.0.0.1:{closed_port}\"\n"
    );

    let path = write_config(&config);
    assert_eq!(run_check(&path, "all-check"), 1);
    assert_eq!(run_check(&path, "any-check"), 0);

    let _ = std::fs::remove_file(&path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}
