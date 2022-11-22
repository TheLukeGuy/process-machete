use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{debug, error, info, warn};
use process_machete::config::ConfigLoadOutcome;
use process_machete::startup::StartupProgramOutcome;
use process_machete::{config, logging, startup};
use std::env;
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

    let args = Args::parse();
    if let Some(Command::Startup { command }) = args.command {
        logging::basic_init(debug).context("failed to initialize logging")?;
        startup_main(command)?;
        return Ok(None);
    }

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
        return Ok(Some(ExitCode::from(-1i8 as u8)));
    };
    debug!("Deserialized config: {:#?}", config);

    process_machete::run(&config)?;
    Ok(None)
}

fn startup_main(command: StartupCommand) -> Result<()> {
    let exe_path =
        env::current_exe().context("failed to get the path of the current running executable")?;
    let (outcome, success_verb) = match command {
        StartupCommand::Add => {
            let outcome =
                startup::add_program(&exe_path).context("failed to add the startup program")?;
            (outcome, "Added")
        }
        StartupCommand::Remove => {
            let outcome = startup::remove_program(&exe_path)
                .context("failed to remove the startup program")?;
            (outcome, "Removed")
        }
    };

    if let StartupProgramOutcome::Succeeded = outcome {
        info!("{} a startup program!", success_verb);
    }
    Ok(())
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Manage a startup program on supported operating systems
    #[command(hide = !startup::SUPPORTED)]
    Startup {
        #[command(subcommand)]
        command: StartupCommand,
    },
}

#[derive(Subcommand)]
enum StartupCommand {
    /// Start running on operating system startup
    Add,
    /// Stop running on operating system startup
    Remove,
}
