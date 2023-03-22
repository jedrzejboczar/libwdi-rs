//! Example that demonstrates how to rerun application as admin to install drivers
//!
//! The application finds the target USB device and then tries to install drivers
//! for it. If it fails due to missing admin permissions it will re-execute itself
//! with elevated privileges using the "runas" action.
//!
//! Further improvements would be to pass information about the selected device
//! to the spawned elevated process, either directly via lpParameters or leveraging
//! IPC, e.g. by creating Windows named pipe and passing it via lpParameters.

use std::env;
use std::io::{Write, BufRead};

use windows::core::*;
use windows::Win32::Foundation;
use windows::Win32::System::Threading;
use windows::Win32::System::WindowsProgramming::INFINITE;
use windows::Win32::UI::Shell;
use windows::Win32::UI::WindowsAndMessaging;

use libwdi as wdi;

const VID: u16 = 0x0483;
const PID: u16 = 0xdf11;
const VENDOR: &str = "Example vendor";
const DRIVER_PATH: &str = "C:\\usb_driver";
const INF_NAME: &str = "MyDeviceWinUSB.inf";

// Get executable for this program, in general it should be better to use a fixed known
// path instead of env::current_exe.
fn get_current_exe() -> std::path::PathBuf {
    env::current_exe()
        .expect("Could not get path to current executable")
}

fn se_err_string(err: u32) -> String {
    match err {
        Shell::SE_ERR_FNF => "File not found.".into(),
        Shell::SE_ERR_PNF => "Path not found.".into(),
        Shell::SE_ERR_ACCESSDENIED => "Access denied.".into(),
        Shell::SE_ERR_OOM => "Out of memory.".into(),
        Shell::SE_ERR_DLLNOTFOUND => "Dynamic-link library not found.".into(),
        Shell::SE_ERR_SHARE => "Cannot share an open file.".into(),
        Shell::SE_ERR_ASSOCINCOMPLETE => "File association information not complete.".into(),
        Shell::SE_ERR_DDETIMEOUT => "DDE operation timed out.".into(),
        Shell::SE_ERR_DDEFAIL => "DDE operation failed.".into(),
        Shell::SE_ERR_DDEBUSY => "DDE operation is busy.".into(),
        Shell::SE_ERR_NOASSOC => "File association not available.".into(),
        _ => format!("Unexpected SE_ERR_* code: {}", err),
    }
}

// Re-run this executable with escalated privileges
// See:
// https://users.rust-lang.org/t/hi-guys-how-do-i-trigger-a-new-process-to-be-run-as-admin/60788/4
// https://stackoverflow.com/a/17638969
fn rerun_as_admin() {
    // By using ShellExecuteEx with SHELLEXECUTEINFOW we can wait for the process to complete
    let exe = HSTRING::from(get_current_exe().to_str().unwrap());
    let show = WindowsAndMessaging::SW_NORMAL; // SW_HIDE to run in background

    let mut exec_info = Shell::SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<Shell::SHELLEXECUTEINFOW>() as u32,
        fMask: Shell::SEE_MASK_NOCLOSEPROCESS, // hProcess will receive process handle
        hwnd: Foundation::HWND(0),
        lpVerb: w!("runas"), // run [a]dmini[s]trator
        lpFile: PCWSTR::from_raw(exe.as_ptr()),
        lpParameters: w!(""),
        lpDirectory: PCWSTR::null(),
        nShow: show.0 as i32,
        ..Default::default()
    };

    // With SEE_MASK_NOCLOSEPROCESS hInstApp is set to >=32 on success or SE_ERR_XXX on failure
    unsafe {
        if !Shell::ShellExecuteExW(&mut exec_info).as_bool() {
            panic!("ShellExecuteExW failed");
        }
    }
    if let err @ 0..=31 = exec_info.hInstApp.0 as u32 {
        panic!("Failed to rerun as administrator: {}", se_err_string(err));
    } else if exec_info.hProcess.is_invalid() {
        panic!("No process was spawned");
    }

    // Wait for the process and clean up
    unsafe {
        Threading::WaitForSingleObject(exec_info.hProcess, INFINITE);
        Foundation::CloseHandle(exec_info.hProcess);
    }

    // Make sure it's not dropped earlier
    drop(exe);
}

fn main() {
    wdi::set_log_level(wdi::LogLevel::Info).unwrap();

    let devices = wdi::CreateListOptions::new()
        .list_all(true)
        .create_list()
        .expect("Failed to list USB devices");

    let mut candidates: Vec<_> = devices.iter()
        .filter(|dev| dev.vid() == VID && dev.pid() == PID)
        .collect();

    println!("Found candidate devices:");
    for dev in candidates.iter() {
        let composite = if dev.is_composite() {
            format!(" composite[{}]", dev.mi().map(|v| v.get()).unwrap_or(0))
        } else {
            "".to_string()
        };
        println!(" => {:04x}:{:04x}{}", dev.vid(), dev.pid(), composite);
        println!("    desc: {}", dev.desc());
        println!("    driver: {}", dev.driver().unwrap_or("-".into()));
        println!("    device ID: {}", dev.device_id().unwrap_or("-".into()));
        println!("    hardware ID: {}", dev.hardware_id().unwrap_or("-".into()));
        println!("    compatible ID: {}", dev.compatible_id().unwrap_or("-".into()));
        println!("    upper filter: {}", dev.upper_filter().unwrap_or("-".into()));
        println!("    driver version: {}", dev.upper_filter().unwrap_or("-".into()));
    }

    if let Some(dev) = candidates.first_mut() {
        println!("Using first device: {:04x}:{:04x}", dev.vid(), dev.pid());
        println!("Continue? [yN] ");
        std::io::stdout().lock().flush().unwrap();

        let mut input = std::io::stdin().lock();
        let mut answer = String::new();
        input.read_line(&mut answer).unwrap();
        let answer = answer.trim().to_lowercase();

        if answer == "y" || answer == "yes" {
            println!("\nInstalling ...");

            let driver = wdi::PrepareDriverOptions::new()
                .driver_type(wdi::DriverType::WinUsb)
                .vendor_name(VENDOR).unwrap()
                .prepare_driver(dev, DRIVER_PATH, INF_NAME)
                .expect("Failed to prepare driver");

            let result = driver.install_driver();

            match result {
                Ok(_) => {},
                Err(wdi::Error::NeedsAdmin) => {
                    println!("Re-running as administrator ...");
                    rerun_as_admin();
                    println!("Done");
                },
                err => err.expect("Failed to install driver"),
            }
        } else {
            println!("\nAborting");
        }
    }
}
