use anyhow::{Context, Result};
use log::{debug, error, info, log_enabled, warn, Level, LevelFilter};
use process_machete::config::{Config, LoggingConfig};
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{convert, env, fs};

const DEFAULT_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/config.toml"
));

fn main() -> ExitCode {
    match inner_main() {
        Err(error) => {
            if log_enabled!(Level::Error) {
                error!("Error: {:?}", error);
            } else {
                eprintln!("Error: {:?}", error);
            }
            ExitCode::FAILURE
        }
        Ok(Some(exit_code)) => exit_code,
        Ok(None) => ExitCode::SUCCESS,
    }
}

fn inner_main() -> Result<Option<ExitCode>> {
    let debug = cfg!(debug_assertions);

    let (config_dir_path, config_dir_explanation) =
        config_dir_and_explanation(debug).context("failed to get the config directory")?;
    let config = load_config(&config_dir_path).context("failed to load the config")?;

    init_logging(debug, &config_dir_path, config.logging_config())
        .context("failed to initialize logging")?;
    if debug {
        warn!("Debug mode is enabled. Things might behave slightly differently!");
    }

    let ConfigLoadOutcome::Loaded(config) = config else {
        info!(
            "A default config.toml file has been created in {}. Configure it!",
            config_dir_explanation
        );
        return Ok(Some(ExitCode::FAILURE));
    };
    debug!("Deserialized config: {:#?}", config);

    process_machete::run(&config)?;
    Ok(None)
}

fn init_logging(debug: bool, config_dir_path: &Path, config: Option<&LoggingConfig>) -> Result<()> {
    let config = config.unwrap_or(&LoggingConfig {
        log_to_file: false,
        always_debug: false,
    });
    let level = if debug || config.always_debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let term_logger = TermLogger::new(
        level,
        ConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    let loggers: Vec<Box<dyn SharedLogger>> = if config.log_to_file {
        let file = File::create(config_dir_path.join("latest_log.txt"))
            .context("failed to create the log file")?;
        let write_logger = WriteLogger::new(
            level,
            ConfigBuilder::new()
                .set_thread_level(LevelFilter::Off)
                .set_target_level(LevelFilter::Off)
                .set_time_offset_to_local()
                .unwrap_or_else(convert::identity)
                .build(),
            file,
        );

        vec![term_logger, write_logger]
    } else {
        vec![term_logger]
    };

    CombinedLogger::init(loggers).context("failed to initialize the logger")
}

enum ConfigLoadOutcome {
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

fn load_config(config_dir_path: &Path) -> Result<ConfigLoadOutcome> {
    let config_path = config_dir_path.join("config.toml");
    debug!("Config path: {}", config_path.display());

    if !config_path.exists() {
        fs::write(&config_path, DEFAULT_CONFIG).with_context(|| {
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

fn config_dir_and_explanation(debug: bool) -> Result<(PathBuf, &'static str)> {
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
