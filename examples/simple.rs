use std::env;
use std::io::{Write as _, BufRead};

use windows::core::*;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::Foundation::HWND;
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD;

use libwdi as wdi;

// https://users.rust-lang.org/t/hi-guys-how-do-i-trigger-a-new-process-to-be-run-as-admin/60788/4
fn rerun_as_admin() {
    let exe = env::current_exe()
        .expect("Could not get path to current executable");
    let result = unsafe {
        ShellExecuteW(HWND(0), &HSTRING::from("runas"), &HSTRING::from(exe.as_os_str()), PCWSTR::null(), PCWSTR::null(), SHOW_WINDOW_CMD(1))
    };
    if result.0 < 32 {
        panic!("Failed to run as administrator");
    }
}

fn main() {
    wdi::set_log_level(wdi::LogLevel::Debug).unwrap();

    let devices = wdi::CreateListOptions::new()
        .list_all(true)
        .create_list()
        .expect("Failed to list USB devices");

    let (vid, pid) = (0x0483, 0xdf11);
    let mut candidates: Vec<_> = devices.iter()
        .filter(|dev| dev.vid() == vid && dev.pid() == pid)
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
                .vendor_name("Example vendor").unwrap()
                .prepare_driver(dev, "C:\\usb_driver", "STM32BootloaderWinUSB.inf")
                .expect("Failed to prepare driver");

            let result = driver.install_driver();

            match result {
                Ok(_) => {},
                Err(wdi::Error::NeedsAdmin) => rerun_as_admin(),
                err => err.expect("Failed to install driver"),
            }
        } else {
            println!("\nAborting");
        }
    }
}
