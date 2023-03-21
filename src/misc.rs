use std::ffi;

use libwdi_sys as wdi;

use crate::enums::{LogLevel, check_error, Result};

pub fn get_vendor_name(vid: u16) -> Option<&'static str> {
    let name = unsafe { wdi::wdi_get_vendor_name(vid) };
    if name.is_null() {
        None
    } else {
        let name = unsafe { ffi::CStr::from_ptr(name) };
        // libwdi guarantees UTF-8 strings
        Some(name.to_str().unwrap())
    }
}

pub fn get_wdf_version() -> std::os::raw::c_int {
    unsafe {
        wdi::wdi_get_wdf_version()
    }
}

pub fn set_log_level(level: LogLevel) -> Result<()> {
    unsafe {
        check_error(wdi::wdi_set_log_level(level.to_ffi()))
    }
}

pub unsafe fn register_logger(hwnd: wdi::HWND, message_id: wdi::UINT, buff_size: wdi::DWORD) -> Result<()> {
    check_error(wdi::wdi_register_logger(hwnd, message_id, buff_size))
}

pub unsafe fn unregister_logger(hwnd: wdi::HWND) -> Result<()> {
    check_error(wdi::wdi_unregister_logger(hwnd))
}
