use anyhow::{Context, Result};
use log::debug;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs};

const DEFAULT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/config.toml"
));

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub killing: KillingConfig,
    pub logging: LoggingConfig,
    pub processes: Vec<ProcessConfig>,
}

#[derive(Debug, Default, Deserialize)]
pub struct KillingConfig {
    #[serde(with = "humantime_serde")]
    pub max_wait_time: Duration,
    #[serde(with = "humantime_serde")]
    pub refresh_wait_time: Duration,
    #[serde(with = "humantime_serde")]
    pub kill_wait_time: Duration,
    pub kill_gracefully: bool,
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

pub enum ConfigLoadOutcome {
    Loaded(Config),
    Created,
}

impl ConfigLoadOutcome {
    pub fn logging_config(&self) -> Option<&LoggingConfig> {
        match self {
            ConfigLoadOutcome::Loaded(config) => Some(&config.logging),
            ConfigLoadOutcome::Created => None,
        }
    }
}

pub fn load(config_dir_path: &Path) -> Result<ConfigLoadOutcome> {
    let config_path = config_dir_path.join("config.toml");
    debug!("Config path: {}", config_path.display());

    if !config_path.exists() {
        fs::write(&config_path, DEFAULT).with_context(|| {
            format!(
                "failed to write the default config to {}",
                config_path.display()
            )
        })?;
        return Ok(ConfigLoadOutcome::Created);
    }

    Config::from_path(&config_path)
        .map(ConfigLoadOutcome::Loaded)
        .with_context(|| {
            format!(
                "failed to load from the config file at {}",
                config_path.display()
            )
        })
}

pub fn dir_and_explanation(debug: bool) -> Result<(PathBuf, &'static str)> {
    if debug {
        let current_dir_path = env::current_dir()
            .context("failed to get the path of the current working directory")?;
        Ok((current_dir_path, "the current folder"))
    } else {
        let exe_dir_path = env::current_exe()
            .context("failed to get the path of the current running executable")?
            .parent()
            .context("the executable path has no parent")?
            .to_path_buf();
        Ok((exe_dir_path, "the same folder as this executable"))
    }
}
