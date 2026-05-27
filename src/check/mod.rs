mod tcp;

use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug)]
pub enum ProbeResult {
    Ok,
    ConnectRefused,
    Timeout,
    OtherErr(std::io::Error),
}

#[derive(Debug)]
pub enum AggregateFailure {
    Failed,
    Timeout,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    All,
    Any,
}

pub fn run_tcp_check(
    targets: &[SocketAddr],
    timeout: Duration,
    mode: Mode,
    verbose: bool,
) -> Result<(), AggregateFailure> {
    match mode {
        Mode::All => run_all_targets(targets, timeout, verbose),
        Mode::Any => run_any_target(targets, timeout, verbose),
    }
}

fn run_all_targets(
    targets: &[SocketAddr],
    timeout: Duration,
    verbose: bool,
) -> Result<(), AggregateFailure> {
    let mut saw_non_timeout_failure = false;

    for target in targets {
        let result = tcp::probe_tcp(*target, timeout);
        log_result(*target, &result, verbose);

        match result {
            ProbeResult::Ok => {}
            ProbeResult::Timeout => return Err(AggregateFailure::Timeout),
            ProbeResult::ConnectRefused | ProbeResult::OtherErr(_) => {
                saw_non_timeout_failure = true;
            }
        }
    }

    if saw_non_timeout_failure {
        Err(AggregateFailure::Failed)
    } else {
        Ok(())
    }
}

fn run_any_target(
    targets: &[SocketAddr],
    timeout: Duration,
    verbose: bool,
) -> Result<(), AggregateFailure> {
    let mut saw_timeout = false;
    let mut saw_non_timeout_failure = false;

    for target in targets {
        let result = tcp::probe_tcp(*target, timeout);
        log_result(*target, &result, verbose);

        match result {
            ProbeResult::Ok => return Ok(()),
            ProbeResult::Timeout => saw_timeout = true,
            ProbeResult::ConnectRefused | ProbeResult::OtherErr(_) => {
                saw_non_timeout_failure = true;
            }
        }
    }

    if saw_non_timeout_failure {
        Err(AggregateFailure::Failed)
    } else if saw_timeout {
        Err(AggregateFailure::Timeout)
    } else {
        Err(AggregateFailure::Failed)
    }
}

fn log_result(target: SocketAddr, result: &ProbeResult, verbose: bool) {
    if !verbose {
        return;
    }

    match result {
        ProbeResult::Ok => eprintln!("{target}: ok"),
        ProbeResult::ConnectRefused => eprintln!("{target}: connection refused"),
        ProbeResult::Timeout => eprintln!("{target}: timeout"),
        ProbeResult::OtherErr(err) => eprintln!("{target}: {err}"),
    }
}
