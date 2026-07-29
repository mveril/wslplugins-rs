use std::ffi::CStr;
use std::fmt;
use std::str::Utf8Error;

/// The JSON image inspection document supplied to a WSLC hook.
#[repr(transparent)]
pub struct WSLCImageInformation(CStr);

impl WSLCImageInformation {
    const fn from_c_str(value: &CStr) -> &Self {
        // SAFETY: both types are transparent wrappers around the same C string.
        unsafe { &*(std::ptr::from_ref(value) as *const Self) }
    }

    /// Returns the inspection document as UTF-8 JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is not valid UTF-8.
    #[inline]
    pub const fn json(&self) -> Result<&str, Utf8Error> {
        self.0.to_str()
    }

    /// Returns the native C string.
    #[must_use]
    #[inline]
    pub const fn as_c_str(&self) -> &CStr {
        &self.0
    }
}

impl<'a> From<&'a CStr> for &'a WSLCImageInformation {
    #[inline]
    fn from(value: &'a CStr) -> Self {
        WSLCImageInformation::from_c_str(value)
    }
}

impl fmt::Debug for WSLCImageInformation {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
