use std::ffi::CStr;
use std::fmt;
use std::str::Utf8Error;

/// A container or image identifier supplied by WSLC.
#[repr(transparent)]
pub struct WSLCObjectId(CStr);

impl WSLCObjectId {
    const fn from_c_str(value: &CStr) -> &Self {
        // SAFETY: both types are transparent wrappers around the same C string.
        unsafe { &*(std::ptr::from_ref(value) as *const Self) }
    }

    /// Returns the identifier as UTF-8.
    ///
    /// # Errors
    ///
    /// Returns an error if WSLC supplied an identifier that is not valid UTF-8.
    #[inline]
    pub const fn to_str(&self) -> Result<&str, Utf8Error> {
        self.0.to_str()
    }

    /// Returns the native C string.
    #[must_use]
    #[inline]
    pub const fn as_c_str(&self) -> &CStr {
        &self.0
    }
}

impl<'a> From<&'a CStr> for &'a WSLCObjectId {
    #[inline]
    fn from(value: &'a CStr) -> Self {
        WSLCObjectId::from_c_str(value)
    }
}

impl fmt::Debug for WSLCObjectId {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
