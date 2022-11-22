use anyhow::{Context, Result};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::Path;

pub const SUPPORTED: bool = cfg!(windows);

pub fn add_item(exe_path: impl AsRef<Path>) -> Result<StartupItemOutcome> {
    let exe_path = prepare_exe_path(exe_path).context("failed to prepare the executable path")?;
    add_item_inner(&exe_path)
}

pub fn remove_item(exe_path: impl AsRef<Path>) -> Result<StartupItemOutcome> {
    let exe_path = prepare_exe_path(exe_path).context("failed to prepare the executable path")?;
    remove_item_inner(&exe_path)
}

pub enum StartupItemOutcome {
    Unsupported,
    Succeeded,
}

fn prepare_exe_path(exe_path: impl AsRef<Path>) -> Result<OsString> {
    let exe_path = exe_path.as_ref();
    let os_string = if !exe_path.is_absolute() {
        fs::canonicalize(exe_path)
            .context("failed to canonicalize the executable path")?
            .as_os_str()
            .to_os_string()
    } else {
        exe_path.as_os_str().to_os_string()
    };
    Ok(os_string)
}

#[cfg(windows)]
const REGISTRY_KEY_NAME: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(windows)]
const REGISTRY_VALUE_NAME: &str = "Process Machete";

#[cfg(windows)]
fn add_item_inner(exe_path: &OsStr) -> Result<StartupItemOutcome> {
    use winapi::um::winnt;
    use winapi::um::winreg;

    let exe_path = encode_wide_str(exe_path);
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
            exe_path.as_ptr() as *const _,
            (exe_path.len() * 2) as _,
        )
    };
    check_status(result).context("failed to set the registry value")?;

    Ok(StartupItemOutcome::Succeeded)
}

#[cfg(windows)]
fn remove_item_inner(_exe_path: &OsStr) -> Result<StartupItemOutcome> {
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

    Ok(StartupItemOutcome::Succeeded)
}

#[cfg(windows)]
fn encode_wide_str(os_str: impl AsRef<OsStr>) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    os_str
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
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
fn add_item_inner(_exe_path: &OsStr) -> Result<StartupItemOutcome> {
    log::error!("You must add the startup item manually on your operating system! Sorry :(");
    Ok(StartupItemOutcome::Unsupported)
}

#[cfg(not(windows))]
fn remove_item_inner(_exe_path: &OsStr) -> Result<StartupItemOutcome> {
    log::error!("You must remove the startup item manually on your operating system! Sorry :(");
    Ok(StartupItemOutcome::Unsupported)
}
