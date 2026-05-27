use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_with_config(contents: &str, subcommand: &[&str]) -> i32 {
    let dir = std::env::temp_dir().join(format!(
        "socketup-tests-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));

    let _ = fs::create_dir_all(&dir);
    let path = dir.join("socketup.yaml");
    let _ = fs::write(&path, contents);

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_socketup"));
    cmd.arg("--config").arg(&path);
    for arg in subcommand {
        cmd.arg(arg);
    }

    let status = cmd.status().expect("socketup must run");
    let _ = fs::remove_file(&path);
    let _ = fs::remove_dir(&dir);

    status.code().unwrap_or(1)
}

#[test]
fn hostname_targets_return_validation_exit_code() {
    for target in ["localhost:8080", "example.com:80", "foo:1234"] {
        let yaml = format!(
            "checks:\n  a:\n    type: tcp\n    targets:\n      - \"{}\"\n",
            target
        );
        let code = run_with_config(&yaml, &["validate"]);
        assert_eq!(code, 5);
    }
}

#[test]
fn invalid_check_name_and_timeout_return_validation_code() {
    let bad_name = "checks:\n  Bad:\n    type: tcp\n    targets:\n      - \"127.0.0.1:8080\"\n";
    assert_eq!(run_with_config(bad_name, &["validate"]), 5);

    let bad_timeout = "defaults:\n  timeout_ms: 0\nchecks:\n  ok:\n    type: tcp\n    targets:\n      - \"127.0.0.1:8080\"\n";
    assert_eq!(run_with_config(bad_timeout, &["validate"]), 5);
}
