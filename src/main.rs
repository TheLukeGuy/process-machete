use anyhow::{Context, Result};
use log::{debug, info, warn, LevelFilter};
use process_machete::config::Config;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use std::path::PathBuf;
use std::{env, fs, process};

const DEFAULT_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/config.toml"
));

fn main() -> Result<()> {
    let debug = cfg!(debug_assertions);
    init_logging(debug).context("failed to initialize logging")?;
    if debug {
        warn!("Debug mode is enabled. Things might behave slightly differently!");
    }

    let config = load_config(debug).context("failed to load the config")?;
    debug!("Deserialized config: {:#?}", config);

    process_machete::run(&config)
}

fn init_logging(debug: bool) -> Result<()> {
    let level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    TermLogger::init(
        level,
        ConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("failed to initialize the logger")
}

fn load_config(debug: bool) -> Result<Config> {
    let (config_dir_path, config_dir_explanation) =
        config_dir_and_explanation(debug).context("failed to get the config directory")?;
    let config_path = config_dir_path.join("config.toml");
    debug!("Config path: {}", config_path.display());

    if !config_path.exists() {
        fs::write(&config_path, DEFAULT_CONFIG).with_context(|| {
            format!(
                "failed to write the default config to {}",
                config_path.display()
            )
        })?;

        info!(
            "A default config.toml file has been created in {}. Configure it!",
            config_dir_explanation
        );
        process::exit(-1);
    }

    Config::from_path(&config_path).with_context(|| {
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
