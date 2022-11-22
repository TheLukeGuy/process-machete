use crate::config::LoggingConfig;
use anyhow::{Context, Result};
use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::convert;
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;

static INITIALIZED: Mutex<bool> = Mutex::new(false);

pub fn init(debug: bool, config_dir_path: &Path, config: Option<&LoggingConfig>) -> Result<()> {
    let config = config.unwrap_or(&LoggingConfig {
        log_to_file: false,
        always_debug: false,
    });
    let level = if debug || config.always_debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let term_logger = term_logger(level);
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

    CombinedLogger::init(loggers).context("failed to initialize the logger")?;

    set_initialized();
    Ok(())
}

pub fn basic_init(debug: bool) -> Result<()> {
    let level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let logger = term_logger(level);
    // TermLogger doesn't have an init method, so we have to initialize with a CombinedLogger
    CombinedLogger::init(vec![logger]).context("failed to initialize the logger")?;

    set_initialized();
    Ok(())
}

fn term_logger(level: LevelFilter) -> Box<TermLogger> {
    TermLogger::new(
        level,
        ConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
}

pub fn initialized() -> bool {
    *INITIALIZED
        .lock()
        .expect("failed to acquire the mutex lock")
}

fn set_initialized() {
    *INITIALIZED
        .lock()
        .expect("failed to acquire the mutex lock") = true;
}
