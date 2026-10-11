use std::ffi::CStr;
use std::fmt;
use std::str::Utf8Error;
use wslc_schema_rs::InspectContainer;

/// The JSON container inspection document supplied to a WSLC hook.
#[repr(transparent)]
pub struct WSLCContainerInformation(CStr);

impl WSLCContainerInformation {
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

    /// Parses the inspection document into its strongly typed WSLC schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is not valid JSON or does not match the
    /// WSLC container inspection schema.
    #[inline]
    pub fn parse(&self) -> Result<InspectContainer, serde_json::Error> {
        serde_json::from_slice(self.0.to_bytes())
    }

    /// Returns the native C string.
    #[must_use]
    #[inline]
    pub const fn as_c_str(&self) -> &CStr {
        &self.0
    }
}

impl<'a> From<&'a CStr> for &'a WSLCContainerInformation {
    #[inline]
    fn from(value: &'a CStr) -> Self {
        WSLCContainerInformation::from_c_str(value)
    }
}

impl fmt::Debug for WSLCContainerInformation {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::WSLCContainerInformation;

    #[test]
    fn parses_container_inspection_document() -> Result<(), Box<dyn std::error::Error>> {
        let information: &WSLCContainerInformation =
            c"{\"Id\":\"container-id\",\"Name\":\"example\"}".into();

        let container = information.parse()?;

        assert_eq!(container.id, "container-id");
        assert_eq!(container.name, "example");
        Ok(())
    }
}
