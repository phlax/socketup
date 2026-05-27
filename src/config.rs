use crate::addr;
use crate::check::Mode;
use crate::exit;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

#[derive(Debug)]
pub struct Config {
    pub checks: BTreeMap<String, Check>,
}

#[derive(Debug)]
pub struct Check {
    pub kind: CheckKind,
}

#[derive(Debug)]
pub enum CheckKind {
    Tcp {
        targets: Vec<SocketAddr>,
        timeout: Duration,
        mode: Mode,
    },
}

#[derive(Debug)]
pub enum ConfigError {
    NotReadable(String),
    Parse(String),
    Validation(String),
}

impl ConfigError {
    pub fn exit_code(&self) -> u8 {
        match self {
            ConfigError::NotReadable(_) => exit::CONFIG_NOT_READABLE,
            ConfigError::Parse(_) => exit::CONFIG_PARSE_ERROR,
            ConfigError::Validation(_) => exit::CONFIG_VALIDATION_ERROR,
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::NotReadable(msg) => write!(f, "config not readable: {msg}"),
            ConfigError::Parse(msg) => write!(f, "config parse error: {msg}"),
            ConfigError::Validation(msg) => write!(f, "config validation error: {msg}"),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    defaults: RawDefaults,
    #[serde(default)]
    checks: BTreeMap<String, RawCheck>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDefaults {
    #[serde(default = "default_timeout_ms")]
    timeout_ms: u64,
    #[serde(default)]
    mode: Mode,
}

impl Default for RawDefaults {
    fn default() -> Self {
        Self {
            timeout_ms: default_timeout_ms(),
            mode: Mode::All,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
enum RawCheck {
    Tcp {
        timeout_ms: Option<u64>,
        mode: Option<Mode>,
        targets: Vec<String>,
    },
}

const NAME_MAX_LEN: usize = 64;

pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let raw = fs::read_to_string(path)
        .map_err(|err| ConfigError::NotReadable(format!("{} ({err})", path.display())))?;

    let parsed_value = serde_yaml::from_str::<serde_yaml::Value>(&raw)
        .map_err(|err| ConfigError::Parse(err.to_string()))?;
    let parsed = serde_yaml::from_value::<RawConfig>(parsed_value)
        .map_err(|err| ConfigError::Validation(err.to_string()))?;
    validate(parsed)
}

fn validate(raw: RawConfig) -> Result<Config, ConfigError> {
    validate_timeout(raw.defaults.timeout_ms, "defaults.timeout_ms")?;

    let mut checks = BTreeMap::new();

    for (name, raw_check) in raw.checks {
        validate_name(&name)?;

        let kind = match raw_check {
            RawCheck::Tcp {
                timeout_ms,
                mode,
                targets,
            } => {
                if targets.is_empty() {
                    return Err(ConfigError::Validation(format!(
                        "check '{name}' has no targets"
                    )));
                }

                let timeout_ms = timeout_ms.unwrap_or(raw.defaults.timeout_ms);
                validate_timeout(timeout_ms, &format!("checks.{name}.timeout_ms"))?;

                let mode = mode.unwrap_or(raw.defaults.mode);
                let mut parsed_targets = Vec::with_capacity(targets.len());
                for target in targets {
                    let parsed = addr::parse_target(&target).map_err(|err| {
                        ConfigError::Validation(format!(
                            "checks.{name}.targets contains invalid target '{target}': {err}"
                        ))
                    })?;
                    parsed_targets.push(parsed);
                }

                CheckKind::Tcp {
                    targets: parsed_targets,
                    timeout: Duration::from_millis(timeout_ms),
                    mode,
                }
            }
        };

        checks.insert(name, Check { kind });
    }

    Ok(Config { checks })
}

fn default_timeout_ms() -> u64 {
    1000
}

fn validate_name(name: &str) -> Result<(), ConfigError> {
    if name.is_empty() {
        return Err(ConfigError::Validation(
            "check name cannot be empty".to_string(),
        ));
    }

    if name.len() > NAME_MAX_LEN {
        return Err(ConfigError::Validation(format!(
            "check name '{name}' exceeds {NAME_MAX_LEN} chars"
        )));
    }

    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(ConfigError::Validation(
            "check name cannot be empty".to_string(),
        ));
    };

    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err(ConfigError::Validation(format!(
            "check name '{name}' must match [a-z0-9][a-z0-9-]*"
        )));
    }

    if !chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
        return Err(ConfigError::Validation(format!(
            "check name '{name}' must match [a-z0-9][a-z0-9-]*"
        )));
    }

    Ok(())
}

fn validate_timeout(timeout_ms: u64, field: &str) -> Result<(), ConfigError> {
    if !(1..=60000).contains(&timeout_ms) {
        return Err(ConfigError::Validation(format!(
            "{field} must be in 1..=60000"
        )));
    }
    Ok(())
}
