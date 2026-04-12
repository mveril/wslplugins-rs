use std::{
    fmt::{self, Debug, Display},
    hash::Hash,
    ptr,
};

#[cfg(feature = "semver")]
mod semver_impl;
pub use semver_impl::SemverConversionError;

/// Represents a WSL version number.
///
/// This struct wraps the `WSLVersion` from the WSL Plugin API and provides
/// safe, idiomatic Rust access to its fields.
/// # Example
/// ```
/// use wslplugins_rs::WSLVersion;
/// let version = WSLVersion::new(2, 0, 0);
/// assert_eq!(version.major(), 2);
/// assert_eq!(version.minor(), 0);
/// assert_eq!(version.revision(), 0);
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WSLVersion(wslpluginapi_sys::WSLVersion);

impl WSLVersion {
    /// Creates a new `WSLVersion` instance.
    /// # Parameters
    /// - `major`: The major version number.
    /// - `minor`: The minor version number.
    /// - `revision`: The revision number.
    /// # Returns
    /// The new `WSLVersion` instance.
    #[must_use]
    #[inline]
    pub const fn new(major: u32, minor: u32, revision: u32) -> Self {
        Self(wslpluginapi_sys::WSLVersion {
            Major: major,
            Minor: minor,
            Revision: revision,
        })
    }

    /// Retrieves the major version number.
    #[must_use]
    #[inline]
    pub const fn major(&self) -> u32 {
        self.0.Major
    }

    /// Set the major version number.
    #[inline]
    pub const fn set_major(&mut self, major: u32) {
        self.0.Major = major;
    }

    /// Retrieves the minor version number.
    #[must_use]
    #[inline]
    pub const fn minor(&self) -> u32 {
        self.0.Minor
    }

    /// Set the minor version number.
    #[inline]
    pub const fn set_minor(&mut self, minor: u32) {
        self.0.Minor = minor;
    }

    /// Retrieves the revision version number.
    #[must_use]
    #[inline]
    pub const fn revision(&self) -> u32 {
        self.0.Revision
    }

    /// Set the revision version number.
    #[inline]
    pub const fn set_revision(&mut self, revision: u32) {
        self.0.Revision = revision;
    }
}

impl From<wslpluginapi_sys::WSLVersion> for WSLVersion {
    #[inline]
    fn from(value: wslpluginapi_sys::WSLVersion) -> Self {
        Self(value)
    }
}

impl From<WSLVersion> for wslpluginapi_sys::WSLVersion {
    #[inline]
    fn from(value: WSLVersion) -> Self {
        value.0
    }
}

impl AsRef<WSLVersion> for wslpluginapi_sys::WSLVersion {
    #[inline]
    fn as_ref(&self) -> &WSLVersion {
        // SAFETY: Converting this reference is safe because `WSLVersion` is
        // `#[repr(transparent)]` over `wslpluginapi_sys::WSLVersion`.
        unsafe { &*ptr::from_ref(self).cast::<WSLVersion>() }
    }
}

impl AsRef<wslpluginapi_sys::WSLVersion> for WSLVersion {
    #[inline]
    fn as_ref(&self) -> &wslpluginapi_sys::WSLVersion {
        &self.0
    }
}

impl Display for WSLVersion {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.revision())
    }
}

impl Debug for WSLVersion {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(WSLVersion))
            .field("major", &self.major())
            .field("minor", &self.minor())
            .field("revision", &self.revision())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_transparence;

    #[test]
    fn test_layouts() {
        test_transparence::<wslpluginapi_sys::WSLVersion, WSLVersion>();
    }
}
