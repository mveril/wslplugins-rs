#[cfg(feature = "semver")]
extern crate semver;
extern crate wslplugins_sys;
#[cfg(feature = "semver")]
use semver::Version;
#[cfg(feature = "semver")]
use std::error::Error;
use std::{fmt, ptr};

#[derive(Hash)]
pub struct WSLVersion<'a>(&'a wslplugins_sys::WSLVersion);

impl fmt::Debug for WSLVersion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(WSLVersion))
            .field("Major", &self.major())
            .field("Minor", &self.minor())
            .field("Revision", &self.revision())
            .finish()
    }
}

impl fmt::Display for WSLVersion<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.revision())
    }
}

impl<'a> From<&'a wslplugins_sys::WSLVersion> for WSLVersion<'a> {
    fn from(value: &'a wslplugins_sys::WSLVersion) -> Self {
        Self(value)
    }
}

impl WSLVersion<'_> {
    pub fn major(&self) -> u32 {
        self.0.Major
    }

    pub fn minor(&self) -> u32 {
        self.0.Minor
    }

    pub fn revision(&self) -> u32 {
        self.0.Revision
    }
}

impl PartialEq for WSLVersion<'_> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.0, other.0) || self.0.eq(other.0)
    }
}

impl Eq for WSLVersion<'_> {}

impl Ord for WSLVersion<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if ptr::eq(self.0, other.0) {
            std::cmp::Ordering::Equal
        } else {
            self.0.cmp(other.0)
        }
    }
}

impl PartialOrd for WSLVersion<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "semver")]
impl From<WSLVersion> for Version {
    fn from(version: WSLVersion) -> Self {
        Version::new(version.major(), version.minor(), version.revision())
    }
}
#[cfg(feature = "semver")]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum WSLVersionError {
    PreReleaseNotEmpty,
    BuildMetadataNotEmpty,
}

#[cfg(feature = "semver")]
impl Error for WSLVersionError {}

#[cfg(feature = "semver")]
impl fmt::Display for WSLVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WSLVersionError::PreReleaseNotEmpty => write!(f, "Pre-release field is not empty"),
            WSLVersionError::BuildMetadataNotEmpty => {
                write!(f, "Build metadata field is not empty")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "semver")]
    use semver::Version;
    use wslplugins_sys::WSLVersion as NativeWSLVersion;

    #[test]
    fn test_from_raw() {
        let native_version = NativeWSLVersion {
            Major: 1,
            Minor: 2,
            Revision: 3,
        };
        let version = WSLVersion::from(&native_version);
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.revision(), 3);
    }

    #[cfg(feature = "semver")]
    #[test]
    fn test_to_semver() {
        let native_version = NativeWSLVersion {
            Major: 1,
            Minor: 2,
            Revision: 3,
        };
        let version = WSLVersion::from(&native_version);
        let semver_version: Version = version.into();
        assert_eq!(semver_version, Version::new(1, 2, 3));
    }

    #[test]
    fn test_comparison() {
        let version1 = NativeWSLVersion {
            Major: 1,
            Minor: 2,
            Revision: 3,
        };
        let version2 = NativeWSLVersion {
            Major: 1,
            Minor: 2,
            Revision: 4,
        };
        let version3 = NativeWSLVersion {
            Major: 2,
            Minor: 0,
            Revision: 0,
        };

        let wsl_version1 = WSLVersion::from(&version1);
        let wsl_version2 = WSLVersion::from(&version2);
        let wsl_version3 = WSLVersion::from(&version3);

        assert!(wsl_version1 < wsl_version2);
        assert!(wsl_version2 < wsl_version3);
        assert!(wsl_version1 < wsl_version3);
    }

    #[test]
    fn test_debug() {
        let version = NativeWSLVersion {
            Major: 1,
            Minor: 2,
            Revision: 3,
        };
        let debug_str = format!("{:?}", WSLVersion::from(&version));
        assert_eq!(debug_str, "WSLVersion { Major: 1, Minor: 2, Revision: 3 }");
    }

    #[test]
    fn test_partial_eq_pointer_equality() {
        let native_version = NativeWSLVersion {
            Major: 1,
            Minor: 2,
            Revision: 3,
        };
        let version1 = WSLVersion::from(&native_version);
        let version2 = WSLVersion::from(&native_version);
        assert_eq!(version1, version2);
    }

    #[test]
    fn test_partial_ord_pointer_equality() {
        let native_version = NativeWSLVersion {
            Major: 1,
            Minor: 2,
            Revision: 3,
        };
        let version1 = WSLVersion::from(&native_version);
        let version2 = WSLVersion::from(&native_version);
        assert_eq!(
            version1.partial_cmp(&version2),
            Some(std::cmp::Ordering::Equal)
        );
    }

    #[cfg(feature = "semver")]
    #[test]
    fn test_display_wslversionerror() {
        assert_eq!(
            format!("{}", WSLVersionError::PreReleaseNotEmpty),
            "Pre-release field is not empty"
        );
        assert_eq!(
            format!("{}", WSLVersionError::BuildMetadataNotEmpty),
            "Build metadata field is not empty"
        );
    }
}
