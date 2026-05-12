use std::{
    fmt::{self, Debug, Display},
    hash::Hash,
    ptr,
    str::FromStr,
};
use strum::IntoEnumIterator;

mod parse_error;
pub use parse_error::WSLVersionParseError;

mod capability;
pub use capability::WSLVersionCapability;

#[cfg(feature = "semver")]
mod semver_impl;
#[cfg(feature = "semver")]
pub use semver_impl::SemverConversionError;

#[cfg(feature = "serde")]
mod serde_impl;

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

    /// Returns `true` when this version is greater than or equal to `required_version`.
    #[must_use]
    #[inline]
    pub const fn is_at_least(&self, required_version: Self) -> bool {
        self.major() > required_version.major()
            || (self.major() == required_version.major()
                && (self.minor() > required_version.minor()
                    || (self.minor() == required_version.minor()
                        && self.revision() >= required_version.revision())))
    }

    /// Returns `true` when this version supports the requested capability.
    #[must_use]
    #[inline]
    pub const fn supports(&self, capability: WSLVersionCapability) -> bool {
        self.is_at_least(capability.required_version())
    }

    /// Iterates over every capability supported by this version, ordered by required version.
    #[inline]
    pub fn capabilities(&self) -> impl Iterator<Item = WSLVersionCapability> + '_ {
        WSLVersionCapability::iter().filter(|capability| self.supports(*capability))
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

impl FromStr for WSLVersion {
    type Err = WSLVersionParseError;

    #[inline]
    #[expect(
        clippy::indexing_slicing,
        reason = "We check the length of `parts` before indexing it, so this is safe."
    )]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if !matches!(parts.len(), 2 | 3) {
            return Err(WSLVersionParseError::InvalidFormat {
                input: s.to_owned(),
            });
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| WSLVersionParseError::InvalidMajor {
                input: s.to_owned(),
            })?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| WSLVersionParseError::InvalidMinor {
                input: s.to_owned(),
            })?;
        let revision = parts
            .get(2)
            .map(|s| s.parse::<u32>())
            .transpose()
            .map_err(|_| WSLVersionParseError::InvalidRevision {
                input: s.to_owned(),
            })?
            .unwrap_or(0);

        Ok(Self::new(major, minor, revision))
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

    #[test]
    fn test_from_str_rejects_invalid_format() {
        let result = "2".parse::<WSLVersion>();

        assert_eq!(
            result,
            Err(WSLVersionParseError::InvalidFormat {
                input: "2".to_owned(),
            })
        );
    }

    #[test]
    fn test_from_str_rejects_invalid_major() {
        let result = "a.0.0".parse::<WSLVersion>();

        assert_eq!(
            result,
            Err(WSLVersionParseError::InvalidMajor {
                input: "a.0.0".to_owned(),
            })
        );
    }

    #[test]
    fn test_from_str_rejects_invalid_minor() {
        let result = "2.a.0".parse::<WSLVersion>();

        assert_eq!(
            result,
            Err(WSLVersionParseError::InvalidMinor {
                input: "2.a.0".to_owned(),
            })
        );
    }

    #[test]
    fn test_from_str_rejects_invalid_revision() {
        let result = "2.0.a".parse::<WSLVersion>();

        assert_eq!(
            result,
            Err(WSLVersionParseError::InvalidRevision {
                input: "2.0.a".to_owned(),
            })
        );
    }

    #[test]
    fn supports_capability_when_version_is_high_enough() {
        let version = WSLVersion::new(2, 1, 2);

        assert!(version.supports(WSLVersionCapability::DistributionRegisteredHook));
    }

    #[test]
    fn rejects_capability_when_version_is_too_low() {
        let version = WSLVersion::new(2, 1, 1);

        assert!(!version.supports(WSLVersionCapability::DistributionRegisteredHook));
    }

    #[test]
    fn capabilities_iterates_supported_capabilities_by_required_version() {
        let version = WSLVersion::new(2, 4, 4);
        let capabilities = version.capabilities().collect::<Vec<_>>();

        assert_eq!(
            capabilities,
            vec![
                WSLVersionCapability::DistributionInitPid,
                WSLVersionCapability::DistributionRegisteredHook,
                WSLVersionCapability::DistributionUnregisteredHook,
                WSLVersionCapability::ExecuteBinaryInDistribution,
                WSLVersionCapability::DistributionFlavor,
                WSLVersionCapability::DistributionVersion,
            ]
        );
    }

    #[test]
    fn capabilities_excludes_capabilities_that_require_newer_versions() {
        let version = WSLVersion::new(2, 1, 2);
        let capabilities = version.capabilities().collect::<Vec<_>>();

        assert_eq!(
            capabilities,
            vec![
                WSLVersionCapability::DistributionInitPid,
                WSLVersionCapability::DistributionRegisteredHook,
                WSLVersionCapability::DistributionUnregisteredHook,
                WSLVersionCapability::ExecuteBinaryInDistribution,
            ]
        );
    }
}
