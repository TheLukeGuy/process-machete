use anyhow::{bail, Context, Result};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub const SUPPORTED: bool = cfg!(windows);

pub fn add_program(exe_path: impl AsRef<Path>) -> Result<StartupProgramOutcome> {
    let exe_path = exe_path.as_ref();
    let exe_path = prepare_exe_path(exe_path).context("failed to prepare the executable path")?;
    add_program_inner(&exe_path)
}

pub fn remove_program(exe_path: impl AsRef<Path>) -> Result<StartupProgramOutcome> {
    let exe_path = exe_path.as_ref();
    let exe_path = prepare_exe_path(exe_path).context("failed to prepare the executable path")?;
    remove_program_inner(&exe_path)
}

pub fn hide_window() -> StartupProgramOutcome {
    // This doesn't really need a separate "inner" function, but might as well for consistency
    // It also keeps all errors in this file if we screw up the platform-specific implementations
    hide_window_inner()
}

pub enum StartupProgramOutcome {
    Unsupported,
    Succeeded,
}

fn prepare_exe_path(exe_path: &Path) -> Result<Cow<OsStr>> {
    let os_string = if !exe_path.is_absolute() {
        fs::canonicalize(exe_path)
            .context("failed to canonicalize the executable path")?
            .as_os_str()
            .to_os_string()
            .into()
    } else {
        exe_path.as_os_str().into()
    };
    Ok(os_string)
}

#[cfg(windows)]
const REGISTRY_KEY_NAME: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(windows)]
const REGISTRY_VALUE_NAME: &str = "Process Machete";

#[cfg(windows)]
fn add_program_inner(exe_path: &OsStr) -> Result<StartupProgramOutcome> {
    use winapi::um::winnt;
    use winapi::um::winreg;

    let Some(exe_path) = exe_path.to_str() else {
        bail!("the provided executable path contains invalid unicode sequences");
    };
    let command = encode_wide_str(&format!("\"{}\" --startup", exe_path));

    let key_name = encode_wide_str(REGISTRY_KEY_NAME);
    let value_name = encode_wide_str(REGISTRY_VALUE_NAME);

    let mut key = std::ptr::null_mut();
    let result = unsafe {
        winreg::RegCreateKeyExW(
            winreg::HKEY_CURRENT_USER,
            key_name.as_ptr(),
            0,
            std::ptr::null_mut(),
            winnt::REG_OPTION_NON_VOLATILE,
            winnt::KEY_SET_VALUE,
            std::ptr::null_mut(),
            &mut key,
            std::ptr::null_mut(),
        )
    };
    check_status(result).context("failed to create the registry key")?;

    let result = unsafe {
        winreg::RegSetValueExW(
            key,
            value_name.as_ptr(),
            0,
            winnt::REG_SZ,
            command.as_ptr() as *const _,
            (command.len() * 2) as _,
        )
    };
    check_status(result).context("failed to set the registry value")?;

    Ok(StartupProgramOutcome::Succeeded)
}

#[cfg(windows)]
fn remove_program_inner(_exe_path: &OsStr) -> Result<StartupProgramOutcome> {
    use winapi::um::winreg;

    let key_name = encode_wide_str(REGISTRY_KEY_NAME);
    let value_name = encode_wide_str(REGISTRY_VALUE_NAME);

    let result = unsafe {
        winreg::RegDeleteKeyValueW(
            winreg::HKEY_CURRENT_USER,
            key_name.as_ptr(),
            value_name.as_ptr(),
        )
    };
    check_status(result).context("failed to delete the registry value")?;

    Ok(StartupProgramOutcome::Succeeded)
}

#[cfg(windows)]
fn hide_window_inner() -> StartupProgramOutcome {
    use winapi::um::winuser;

    let window = unsafe { winapi::um::wincon::GetConsoleWindow() };
    if !window.is_null() {
        unsafe {
            winuser::ShowWindow(window, winuser::SW_HIDE);
        }
    }
    StartupProgramOutcome::Succeeded
}

#[cfg(windows)]
fn encode_wide_str(str: &str) -> Vec<u16> {
    str.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn check_status(status: winapi::um::winreg::LSTATUS) -> Result<()> {
    if status == (winapi::shared::winerror::ERROR_SUCCESS as _) {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(status).into())
    }
}

#[cfg(not(windows))]
fn add_program_inner(_exe_path: &OsStr) -> Result<StartupProgramOutcome> {
    log::error!("You must add the startup program manually on your operating system! Sorry :(");
    Ok(StartupProgramOutcome::Unsupported)
}

#[cfg(not(windows))]
fn remove_program_inner(_exe_path: &OsStr) -> Result<StartupProgramOutcome> {
    log::error!("You must remove the startup program manually on your operating system! Sorry :(");
    Ok(StartupProgramOutcome::Unsupported)
}

#[cfg(not(windows))]
fn hide_window_inner() -> StartupProgramOutcome {
    StartupProgramOutcome::Unsupported
}
