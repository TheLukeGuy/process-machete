use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use process_machete::config::ConfigLoadOutcome;
use process_machete::{config, logging};
use std::process::ExitCode;

fn main() -> ExitCode {
    match inner_main() {
        Err(error) => {
            if logging::initialized() {
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
        config::dir_and_explanation(debug).context("failed to get the config directory")?;
    let config = config::load(&config_dir_path).context("failed to load the config")?;

    logging::init(debug, &config_dir_path, config.logging_config())
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
