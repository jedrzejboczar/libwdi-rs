use std::{str::Utf8Error, ffi};

use thiserror::Error;
use libwdi_sys as wdi;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Input/output error")]
    Io,
    #[error("Invalid parameter")]
    InvalidParam,
    #[error("Access denied (insufficient permissions)")]
    Access,
    #[error("No such device (it may have been disconnected)")]
    NoDevice,
    #[error("Entity not found")]
    NotFound,
    #[error("Resource busy, or API call already running")]
    Busy,
    #[error("Operation timed out")]
    Timeout,
    #[error("Overflow")]
    Overflow,
    #[error("Another installation is pending")]
    PendingInstallation,
    #[error("System call interrupted (perhaps due to signal)")]
    Interrupted,
    #[error("Could not acquire resource (Insufficient memory, etc)")]
    Resource,
    #[error("Operation not supported or unimplemented on this platform")]
    NotSupported,
    #[error("Entity already exists")]
    Exists,
    #[error("Cancelled by user")]
    UserCancel,
    #[error("Couldn't run installer with required privileges")]
    NeedsAdmin,
    #[error("Attempted to run the 32 bit installer on 64 bit")]
    WOW64,
    #[error("Bad inf syntax")]
    InfSyntax,
    #[error("Missing cat file")]
    CatMissing,
    #[error("System policy prevents the installation of unsigned drivers")]
    Unsigned,
    #[error("Other error")]
    Other,
    #[error("Unexpected libwdi error code {0}, this may indicate FFI issues")]
    Unexpected(wdi::wdi_error::Type),
    #[error("Internal error in the wrapper or library broken it's contract")]
    Internal,
    #[error("C to Rust string conversion error")]
    Utf8(#[from] Utf8Error),
    #[error("Rust to C string conversion error")]
    Nul(#[from] ffi::NulError),
}

pub fn check_error(code: wdi::wdi_error::Type) -> Result<()> {
    match code {
        wdi::wdi_error::WDI_SUCCESS => Ok(()),
        wdi::wdi_error::WDI_ERROR_IO => Err(Error::Io),
        wdi::wdi_error::WDI_ERROR_INVALID_PARAM => Err(Error::InvalidParam),
        wdi::wdi_error::WDI_ERROR_ACCESS => Err(Error::Access),
        wdi::wdi_error::WDI_ERROR_NO_DEVICE => Err(Error::NoDevice),
        wdi::wdi_error::WDI_ERROR_NOT_FOUND => Err(Error::NotFound),
        wdi::wdi_error::WDI_ERROR_BUSY => Err(Error::Busy),
        wdi::wdi_error::WDI_ERROR_TIMEOUT => Err(Error::Timeout),
        wdi::wdi_error::WDI_ERROR_OVERFLOW => Err(Error::Overflow),
        wdi::wdi_error::WDI_ERROR_PENDING_INSTALLATION => Err(Error::PendingInstallation),
        wdi::wdi_error::WDI_ERROR_INTERRUPTED => Err(Error::Interrupted),
        wdi::wdi_error::WDI_ERROR_RESOURCE => Err(Error::Resource),
        wdi::wdi_error::WDI_ERROR_NOT_SUPPORTED => Err(Error::NotSupported),
        wdi::wdi_error::WDI_ERROR_EXISTS => Err(Error::Exists),
        wdi::wdi_error::WDI_ERROR_USER_CANCEL => Err(Error::UserCancel),
        wdi::wdi_error::WDI_ERROR_NEEDS_ADMIN => Err(Error::NeedsAdmin),
        wdi::wdi_error::WDI_ERROR_WOW64 => Err(Error::WOW64),
        wdi::wdi_error::WDI_ERROR_INF_SYNTAX => Err(Error::InfSyntax),
        wdi::wdi_error::WDI_ERROR_CAT_MISSING => Err(Error::CatMissing),
        wdi::wdi_error::WDI_ERROR_UNSIGNED => Err(Error::Unsigned),
        wdi::wdi_error::WDI_ERROR_OTHER => Err(Error::Other),
        other => Err(Error::Unexpected(other)),
    }
}

/// Type of driver to install
#[derive(Debug, Clone, Copy)]
pub enum DriverType {
    WinUsb,
    LibUsb0,
    LibUsbK,
    Cdc,
    User,
}

impl DriverType {
    pub(crate) fn to_ffi(self) -> wdi::wdi_driver_type::Type {
        match self {
            DriverType::WinUsb => wdi::wdi_driver_type::WDI_WINUSB,
            DriverType::LibUsb0 => wdi::wdi_driver_type::WDI_LIBUSB0,
            DriverType::LibUsbK => wdi::wdi_driver_type::WDI_LIBUSBK,
            DriverType::Cdc => wdi::wdi_driver_type::WDI_CDC,
            DriverType::User => wdi::wdi_driver_type::WDI_USER,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    None,
}

impl LogLevel {
    pub fn to_ffi(self) -> wdi::wdi_log_level::Type {
        match self {
            LogLevel::Debug => wdi::wdi_log_level::WDI_LOG_LEVEL_DEBUG,
            LogLevel::Info => wdi::wdi_log_level::WDI_LOG_LEVEL_INFO,
            LogLevel::Warning => wdi::wdi_log_level::WDI_LOG_LEVEL_WARNING,
            LogLevel::Error => wdi::wdi_log_level::WDI_LOG_LEVEL_ERROR,
            LogLevel::None => wdi::wdi_log_level::WDI_LOG_LEVEL_NONE,
        }
    }
}
