use std::{ffi, ptr};

use libwdi_sys as wdi;

use crate::enums::{LogLevel, check_error, Result, DriverType};

pub struct DriverInfo(pub wdi::tagVS_FIXEDFILEINFO);

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

/// Returns `Some` if driver is supported. For WinUsb/LibUsb0/LibUsbK the info structure is filled,
/// otherwise it is zeroed.
pub fn is_driver_supported(driver_type: DriverType) -> Option<DriverInfo> {
    let mut info: wdi::tagVS_FIXEDFILEINFO = unsafe { std::mem::zeroed() };
    let is_supported = unsafe {
        wdi::wdi_is_driver_supported(driver_type.to_ffi(), &mut info as *mut wdi::tagVS_FIXEDFILEINFO) != 0
    };
    if is_supported {
        Some(DriverInfo(info))
    } else {
        None
    }
}

/// This may fail during string conversion
pub fn is_file_embedded(path: Option<&str>, name: &str) -> Result<bool> {
    let path = path.map(ffi::CString::new).transpose()?;
    let name = ffi::CString::new(name)?;
    let path = path.map_or(ptr::null(), |s| s.as_ptr());
    let result = unsafe {
        wdi::wdi_is_file_embedded(path as *const i8, name.as_ptr() as *const i8)
    };
    Ok(result != 0)
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

pub fn read_logger(buf: &mut [u8]) -> Result<usize> {
    let mut size = 0;
    unsafe {
        check_error(wdi::wdi_read_logger(buf.as_mut_ptr() as *mut i8, buf.len() as u32, &mut size))?
    }
    Ok(size as usize)
}
