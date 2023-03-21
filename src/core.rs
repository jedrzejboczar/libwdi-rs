use std::{ffi, num::{NonZeroU8, NonZeroU64}, borrow::Cow, ptr};

use libwdi_sys as wdi;

use crate::enums::{check_error, Error, Result, DriverType};

/// Builder of options for wdi_create_list
#[derive(Clone)]
pub struct CreateListOptions(wdi::wdi_options_create_list);

/// List of information about USB devices optained from wdi_create_list
pub struct DevicesList(*mut wdi::wdi_device_info);

/// Iterator over devices in the list
pub struct DevicesIter<'a>(Option<DeviceInfo<'a>>);

/// Information related to device installation
pub struct DeviceInfo<'a>(&'a mut wdi::wdi_device_info);

/// Builder of options for wdi_prepare_driver
#[derive(Clone)]
pub struct PrepareDriverOptions {
    opts: wdi::wdi_options_prepare_driver,
    vendor_name: Option<ffi::CString>,
    device_guid: Option<ffi::CString>,
    cert_subject: Option<ffi::CString>,
}

/// Driver files prepared to be installed using wdi_install_driver
pub struct PreparedDriver<'a> {
    dev: &'a mut DeviceInfo<'a>,
    path: ffi::CString,
    inf_name: ffi::CString,
    options: wdi::wdi_options_install_driver,
}

/// Builder of options for wdi_install_trusted_certificate
pub struct InstallCertOptions(wdi::wdi_options_install_cert);

macro_rules! impl_builder_bool {
    ($opts:tt: $( $field:ident ),+ $(,)?) => {
        $(
            pub fn $field(mut self, enabled: bool) -> Self {
                self.$opts.$field = enabled as wdi::BOOL;
                self
            }
         )+
    };
}

impl Default for CreateListOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateListOptions {
    pub const fn new() -> Self {
        Self(wdi::wdi_options_create_list {
            list_all: false as wdi::BOOL,
            list_hubs: false as wdi::BOOL,
            trim_whitespaces: false as wdi::BOOL,
        })
    }

    impl_builder_bool!(0: list_all, list_hubs, trim_whitespaces);

    #[doc(alias = "wdi_create_driver")]
    pub fn create_list(&self) -> Result<DevicesList> {
        DevicesList::new(self.clone())
    }
}

impl DevicesList {
    fn new(mut options: CreateListOptions) -> Result<Self> {
        let mut list: *mut wdi::wdi_device_info = ptr::null_mut();
        unsafe {
            check_error(wdi::wdi_create_list(&mut list, &mut options.0))?;
        }
        if list.is_null() {
            return Err(Error::Internal)
        }
        Ok(Self(list))
    }

    pub fn iter(&self) -> DevicesIter {
        DevicesIter(if self.0.is_null() {
            None
        } else {
            Some(unsafe {
                DeviceInfo::new(&mut *self.0)
            })
        })
    }
}

impl Drop for DevicesList {
    fn drop(&mut self) {
        unsafe {
            check_error(wdi::wdi_destroy_list(self.0)).unwrap();
        }
    }
}

impl<'a> Iterator for DevicesIter<'a> {
    type Item = DeviceInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut info) = self.0.take() {
            let next = info.next();
            let item = Some(info);
            self.0 = next;
            item
        } else {
            None
        }
    }
}

impl<'a> DeviceInfo<'a> {
    fn new(info: &'a mut wdi::wdi_device_info) -> Self {
        Self(info)
    }

    pub fn vid(&self) -> u16 {
        self.0.vid
    }

    pub fn pid(&self) -> u16 {
        self.0.pid
    }

    pub fn is_composite(&self) -> bool {
        self.0.is_composite != 0
    }

    pub fn mi(&self) -> Option<NonZeroU8> {
        NonZeroU8::new(self.0.mi)
    }

    pub fn driver_version(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.0.driver_version)
    }

    fn opt_string(&self, s: *const std::os::raw::c_char) -> Option<Cow<'_, str>> {
        if s.is_null() {
            None
        } else {
            Some(unsafe {
                ffi::CStr::from_ptr(s).to_string_lossy()
            })
        }
    }

    pub fn desc(&self) -> Cow<'_, str> {
        assert!(!self.0.desc.is_null());
        unsafe {
            ffi::CStr::from_ptr(self.0.desc).to_string_lossy()
        }
    }

    pub fn driver(&self) -> Option<Cow<'_, str>> {
        self.opt_string(self.0.driver)
    }

    pub fn device_id(&self) -> Option<Cow<'_, str>> {
        self.opt_string(self.0.device_id)
    }

    pub fn hardware_id(&self) -> Option<Cow<'_, str>> {
        self.opt_string(self.0.hardware_id)
    }

    pub fn compatible_id(&self) -> Option<Cow<'_, str>> {
        self.opt_string(self.0.compatible_id)
    }

    pub fn upper_filter(&self) -> Option<Cow<'_, str>> {
        self.opt_string(self.0.upper_filter)
    }
}

impl<'a> Iterator for DeviceInfo<'a> {
    type Item = DeviceInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.next.is_null() {
            None
        } else {
            Some(unsafe {
                Self::new(&mut *self.0.next)
            })
        }
    }
}

impl PrepareDriverOptions {
    pub fn new() -> Self {
        Self {
            vendor_name: None,
            device_guid: None,
            cert_subject: None,
            opts: wdi::wdi_options_prepare_driver {
                driver_type: wdi::wdi_driver_type::WDI_WINUSB,
                vendor_name: ptr::null_mut(),
                device_guid: ptr::null_mut(),
                disable_cat: false as wdi::BOOL,
                disable_signing: false as wdi::BOOL,
                cert_subject: ptr::null_mut(),
                use_wcid_driver: false as wdi::BOOL,
                external_inf: false as wdi::BOOL,
            }
        }
    }

    impl_builder_bool!(opts: disable_cat, disable_signing, use_wcid_driver, external_inf);

    pub fn driver_type(mut self, typ: DriverType) -> Self {
        self.opts.driver_type = typ.to_ffi();
        self
    }

    pub fn vendor_name(mut self, s: &str) -> Result<Self> {
        self.vendor_name = Some(ffi::CString::new(s)?);
        Ok(self)
    }

    pub fn device_guid(mut self, s: &str) -> Result<Self> {
        self.device_guid = Some(ffi::CString::new(s)?);
        Ok(self)
    }

    pub fn cert_subject(mut self, s: &str) -> Result<Self> {
        self.cert_subject = Some(ffi::CString::new(s)?);
        Ok(self)
    }

    #[doc(alias = "wdi_prepare_driver")]
    pub fn prepare_driver<'a>(mut self, dev: &'a mut DeviceInfo<'a>, path: &str, inf_name: &str) -> Result<PreparedDriver<'a>> {
        let path = ffi::CString::new(path)?;
        let inf_name = ffi::CString::new(inf_name)?;

        // If some optional strings have been provided, pass them in opts.
        // Safety: they should remain valid until after wdi_prepare_driver returns (until self is dropped),
        // if wdi_prepare_driver modifies these strings than we're screwed (but it shouldn't)
        if let Some(s) = &mut self.vendor_name {
            self.opts.vendor_name = s.as_ptr() as *mut i8;
        }
        if let Some(s) = &mut self.device_guid {
            self.opts.device_guid = s.as_ptr() as *mut i8;
        }
        if let Some(s) = &mut self.cert_subject {
            self.opts.cert_subject = s.as_ptr() as *mut i8;
        }

        unsafe {
            check_error(wdi::wdi_prepare_driver(&mut *dev.0, path.as_ptr(), inf_name.as_ptr(), &mut self.opts))?;
        }

        // Make sure that self stays valid until now
        drop(self);

        Ok(PreparedDriver {
            dev,
            path,
            inf_name,
            options: PreparedDriver::DEFAULT_OPTIONS
        })
    }
}

impl Default for PrepareDriverOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PreparedDriver<'a> {
    const DEFAULT_OPTIONS: wdi::wdi_options_install_driver = wdi::wdi_options_install_driver {
        // TODO: try obtaining it using "windows" crate https://stackoverflow.com/a/2620522
        hWnd: ptr::null_mut(),
        install_filter_driver: false as wdi::BOOL,
        pending_install_timeout: 0,
    };

    impl_builder_bool!(options: install_filter_driver);

    pub fn pending_install_timeout(mut self, timeout: u32) -> Self {
        self.options.pending_install_timeout = timeout;
        self
    }

    #[doc(alias = "wdi_install_driver")]
    pub fn install_driver(mut self) -> Result<()> {
        unsafe {
            check_error(wdi::wdi_install_driver(self.dev.0, self.path.as_ptr(), self.inf_name.as_ptr(), &mut self.options))
        }
    }
}

impl InstallCertOptions {
    pub fn new() -> Self {
        Self(wdi::wdi_options_install_cert {
            hWnd: ptr::null_mut(),
            disable_warning: false as wdi::BOOL,
        })
    }

    impl_builder_bool!(0: disable_warning);

    pub fn hwnd(mut self, hwnd: wdi::HWND) -> Self {
        self.0.hWnd = hwnd;
        self
    }

    pub fn install_trusted_certificate(mut self, cert_name: &str) -> Result<()> {
        let cert_name = ffi::CString::new(cert_name)?;
        unsafe {
            check_error(wdi::wdi_install_trusted_certificate(cert_name.as_ptr(), &mut self.0))
        }
    }
}

impl Default for InstallCertOptions {
    fn default() -> Self {
        Self::new()
    }
}
