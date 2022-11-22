use crate::config::{Config, ProcessConfig, ProcessNameMatch};
use anyhow::{bail, Result};
use log::{info, warn};
use std::thread;
use std::time::Instant;
use sysinfo::{Pid, Process, ProcessExt, Signal, System, SystemExt};

pub mod config;
pub mod startup;

pub fn run(config: &Config) -> Result<()> {
    if !system_supported() {
        bail!("this operating system is unsupported");
    }

    let mut processes: Vec<_> = config.processes.iter().map(WatchedProcess::new).collect();
    let start_process_count = processes.len();

    let mut sys = System::new();
    let mut total_kill_count = 0;
    let mut configured_kill_count = 0;

    info!(
        "Started watching for {} {}!",
        start_process_count,
        process_word(start_process_count)
    );

    let start_time = Instant::now();
    while !processes.is_empty() {
        sys.refresh_processes();

        processes.retain_mut(|process| match process.check(config, &sys) {
            ProcessCheckOutcome::NotKilled => true,
            ProcessCheckOutcome::Killed(count) => {
                total_kill_count += count;
                configured_kill_count += 1usize;
                false
            }
        });
        if processes.is_empty() {
            break;
        }

        if !config.killing.max_wait_time.is_zero() {
            let elapsed_time = Instant::now() - start_time;
            if elapsed_time + config.killing.refresh_wait_time >= config.killing.max_wait_time {
                let len_before_purge = processes.len();
                // Keep the ones that have been spawned but are waiting to be killed
                processes.retain(|process| process.found.is_some());

                // If there are fewer processes than before, they didn't all spawn in time
                let we_failed = processes.len() != len_before_purge;
                if processes.is_empty() {
                    if we_failed {
                        warn!("Took too long, surrendering. o7");
                    }
                    break;
                }
                if we_failed {
                    warn!("Took too long, surrendering after spawned processes are killed. o7");
                }
            }
        }
        thread::sleep(config.killing.refresh_wait_time);
    }

    let percent_killed = ((configured_kill_count as f64) / (start_process_count as f64)) * 100.0;
    info!(
        "Done! Killed {} total {}, or {}/{} ({:.00}%) of configured processes.",
        total_kill_count,
        process_word(total_kill_count),
        configured_kill_count,
        start_process_count,
        percent_killed,
    );

    Ok(())
}

struct WatchedProcess<'a> {
    pub config: &'a ProcessConfig,
    pub found: Option<FoundProcess>,
}

struct FoundProcess {
    pub kill_time: Instant,
    pub ids: Vec<Pid>,
}

impl<'a> WatchedProcess<'a> {
    pub fn new(config: &'a ProcessConfig) -> Self {
        Self {
            config,
            found: None,
        }
    }

    pub fn check(&mut self, config: &Config, sys: &System) -> ProcessCheckOutcome {
        if let Some(found) = &self.found {
            if Instant::now() < found.kill_time {
                return ProcessCheckOutcome::NotKilled;
            }

            let processes = found
                .ids
                .iter()
                .filter_map(|&pid| {
                    let process = sys.process(pid);
                    if process.is_none() {
                        warn!("A matching process with pid {} died on its own.", pid);
                    }
                    process
                })
                .collect();

            let kill_count = self.kill(config, &processes);
            return ProcessCheckOutcome::Killed(kill_count);
        }

        let found: Vec<_> = match &self.config.name_match {
            ProcessNameMatch::Exact(name) => sys.processes_by_exact_name(name).collect(),
            ProcessNameMatch::Contains(name) => sys.processes_by_name(name).collect(),
        };
        if found.is_empty() {
            return ProcessCheckOutcome::NotKilled;
        }

        for &process in &found {
            info!("Found: {} (pid {})", process.name(), process.pid());
        }

        let wait_time = self
            .config
            .kill_wait_time
            .unwrap_or(config.killing.kill_wait_time);
        if wait_time.is_zero() {
            let kill_count = self.kill(config, &found);
            return ProcessCheckOutcome::Killed(kill_count);
        }

        let ids = found.into_iter().map(|process| process.pid()).collect();
        self.found = Some(FoundProcess {
            kill_time: Instant::now() + wait_time,
            ids,
        });

        ProcessCheckOutcome::NotKilled
    }

    fn kill(&self, config: &Config, processes: &Vec<&Process>) -> usize {
        let kill_gracefully = self
            .config
            .kill_gracefully
            .unwrap_or(config.killing.kill_gracefully);
        let signal = if !kill_gracefully {
            Signal::Term
        } else {
            Signal::Kill
        };
        let limit = self.config.limit.unwrap_or(processes.len());

        let mut killed = 0;
        for &process in processes {
            if killed >= limit {
                break;
            }

            // `kill_with` returns `None` if the platform doesn't support the given signal
            let success = process.kill_with(signal).unwrap_or_else(|| process.kill());
            if success {
                killed += 1;
                warn!("Killed: {} (pid {})", process.name(), process.pid());
            } else {
                warn!(
                    "Failed to kill process `{}` with pid {}.",
                    process.name(),
                    process.pid()
                );
            }
        }
        killed
    }
}

enum ProcessCheckOutcome {
    NotKilled,
    Killed(usize),
}

fn process_word(processes: usize) -> &'static str {
    if processes == 1 {
        "process"
    } else {
        "processes"
    }
}

fn system_supported() -> bool {
    // This is used to bypass editor inspections that check for constant expressions
    // The value changes depending on which operating system we're compiling for!
    System::IS_SUPPORTED
}
