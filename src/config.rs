use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(with = "humantime_serde")]
    pub max_wait_time: Duration,
    #[serde(with = "humantime_serde")]
    pub refresh_wait_time: Duration,
    #[serde(with = "humantime_serde")]
    pub kill_wait_time: Duration,
    pub kill_gracefully: bool,
    pub logging: LoggingConfig,
    pub processes: Vec<ProcessConfig>,
}

#[derive(Debug, Default, Deserialize)]
pub struct LoggingConfig {
    pub log_to_file: bool,
    pub always_debug: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProcessConfig {
    #[serde(flatten)]
    pub name_match: ProcessNameMatch,
    pub limit: Option<usize>,
    #[serde(default, with = "humantime_serde")]
    pub kill_wait_time: Option<Duration>,
    pub kill_gracefully: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessNameMatch {
    Exact(String),
    Contains(String),
}

impl Config {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read_to_string(&path).with_context(|| {
            format!(
                "failed to read from the config file at {}",
                path.as_ref().display()
            )
        })?;
        toml::from_str(&contents).context("failed to deserialize the config")
    }
}
