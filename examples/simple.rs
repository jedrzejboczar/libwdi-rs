use std::io::{Write as _, BufRead};

use libwdi as wdi;

fn main() {
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

            driver.install_driver()
                .expect("Failed to install driver");
        } else {
            println!("\nAborting");
        }
    }
}
