use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "socketup")]
pub struct Cli {
    #[arg(
        long,
        env = "SOCKETUP_CONFIG",
        default_value = "/etc/socketup/socketup.yaml",
        global = true
    )]
    pub config: PathBuf,

    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Check { name: String },
    Validate,
    Install { dst: PathBuf },
    Version,
}

pub fn target_triple() -> String {
    format!(
        "{}-{}-{}",
        std::env::consts::ARCH,
        target_vendor(),
        target_os_env()
    )
}

fn target_vendor() -> &'static str {
    if cfg!(target_vendor = "unknown") {
        "unknown"
    } else if cfg!(target_vendor = "apple") {
        "apple"
    } else if cfg!(target_vendor = "pc") {
        "pc"
    } else {
        "vendor"
    }
}

fn target_os_env() -> &'static str {
    if cfg!(all(target_os = "linux", target_env = "gnu")) {
        "linux-gnu"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        std::env::consts::OS
    }
}
