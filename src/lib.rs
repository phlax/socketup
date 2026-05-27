pub mod addr;
pub mod check;
pub mod cli;
pub mod config;
pub mod exit;
pub mod install;

use clap::Parser;
use cli::{Cli, Commands};
use std::process::ExitCode;

pub fn main_entry() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();
            return ExitCode::from(exit::BAD_CLI_USAGE);
        }
    };

    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err((code, message, quiet)) => {
            if !quiet {
                eprintln!("{message}");
            }
            ExitCode::from(code)
        }
    }
}

fn run(cli: Cli) -> Result<u8, (u8, String, bool)> {
    match cli.command {
        Commands::Check { name } => {
            let config = config::load_config(&cli.config)
                .map_err(|err| (err.exit_code(), err.to_string(), cli.quiet))?;
            let check = config.checks.get(&name).ok_or_else(|| {
                (
                    exit::CHECK_NOT_FOUND,
                    format!("unknown check '{name}'"),
                    cli.quiet,
                )
            })?;

            let result = match &check.kind {
                config::CheckKind::Tcp {
                    targets,
                    timeout,
                    mode,
                } => check::run_tcp_check(targets, *timeout, *mode, cli.verbose),
            };

            match result {
                Ok(()) => Ok(exit::SUCCESS),
                Err(check::AggregateFailure::Failed) => Ok(exit::CHECK_FAILED),
                Err(check::AggregateFailure::Timeout) => Ok(exit::TIMEOUT),
            }
        }
        Commands::Validate => {
            let config = config::load_config(&cli.config)
                .map_err(|err| (err.exit_code(), err.to_string(), cli.quiet))?;
            if cli.verbose {
                eprintln!("validated {} checks", config.checks.len());
            }
            Ok(exit::SUCCESS)
        }
        Commands::Install { dst } => {
            install::install_self(&dst).map_err(|err| {
                (
                    exit::INTERNAL_ERROR,
                    format!("install failed: {err}"),
                    cli.quiet,
                )
            })?;
            Ok(exit::SUCCESS)
        }
        Commands::Version => {
            println!("{} {}", env!("CARGO_PKG_VERSION"), cli::target_triple());
            Ok(exit::SUCCESS)
        }
    }
}
