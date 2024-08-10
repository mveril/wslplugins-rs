extern crate wslplugins_sys;
#[cfg(feature = "semver")]
extern crate semver;
use std::fmt;
#[cfg(feature = "semver")]
use std::error::Error;
#[cfg(feature = "semver")]
use semver::Version;

#[derive(Eq)]
pub struct WSLVersion(*const wslplugins_sys::WSLVersion);

impl std::hash::Hash for WSLVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.major().hash(state);
        self.minor().hash(state);
        self.revision().hash(state);
    }
}

impl fmt::Debug for WSLVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(WSLVersion))
            .field("Major", &self.major())
            .field("Minor", &self.minor())
            .field("Revision", &self.revision())
            .finish()
    }
}

impl fmt::Display for WSLVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.revision())
    }
}

impl WSLVersion {
    pub fn from_raw(native_version: *const wslplugins_sys::WSLVersion) -> Self {
        WSLVersion(native_version)
    }

    pub fn major(&self) -> u32 {
        unsafe { (*self.0).Major }
    }

    pub fn minor(&self) -> u32 {
        unsafe { (*self.0).Minor }
    }

    pub fn revision(&self) -> u32 {
        unsafe { (*self.0).Revision }
    }
}

impl PartialEq for WSLVersion {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 || (
            self.major() == other.major() && self.minor() == other.minor() && self.revision() == other.revision()
        )
    }
}

impl Ord for WSLVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.0 == other.0 {
            std::cmp::Ordering::Equal
        } else {
            self.major()
                .cmp(&other.major())
                .then_with(|| self.minor().cmp(&other.minor()))
                .then_with(|| self.revision().cmp(&other.revision()))
        }
    }
}

impl PartialOrd for WSLVersion {
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum WSLVersionError {
    PreReleaseNotEmpty,
    BuildMetadataNotEmpty,
}

#[cfg(feature = "semver")]
impl Error for WSLVersionError {}

impl fmt::Display for WSLVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WSLVersionError::PreReleaseNotEmpty => write!(f, "Pre-release field is not empty"),
            WSLVersionError::BuildMetadataNotEmpty => write!(f, "Build metadata field is not empty"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wslplugins_sys::WSLVersion as NativeWSLVersion;
    #[cfg(feature = "semver")]
    use semver::Version;

    #[test]
    fn test_from_raw() {
        let native_version = NativeWSLVersion {
            Major: 1,
            Minor: 2,
            Revision: 3,
        };
        let version = WSLVersion::from_raw(&native_version as *const wslplugins_sys::WSLVersion);
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
        let version = WSLVersion::from_raw(&native_version as *const wslplugins_sys::WSLVersion);
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

        let wsl_version1 = WSLVersion::from_raw(&version1 as *const _ as *const wslplugins_sys::WSLVersion);
        let wsl_version2 = WSLVersion::from_raw(&version2 as *const _ as *const wslplugins_sys::WSLVersion);
        let wsl_version3 = WSLVersion::from_raw(&version3 as *const _ as *const wslplugins_sys::WSLVersion);

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
        let debug_str = format!("{:?}", WSLVersion::from_raw(&version as *const _ as *const wslplugins_sys::WSLVersion));
        assert_eq!(debug_str, "WSLVersion { Major: 1, Minor: 2, Revision: 3 }");
    }

    #[test]
    fn test_partial_eq_pointer_equality() {
        let native_version = NativeWSLVersion {
            Major: 1,
            Minor: 2,
            Revision: 3,
        };
        let version1 = WSLVersion::from_raw(&native_version as *const _ as *const wslplugins_sys::WSLVersion);
        let version2 = WSLVersion::from_raw(&native_version as *const _ as *const wslplugins_sys::WSLVersion);
        assert_eq!(version1, version2);
    }

    #[test]
    fn test_partial_ord_pointer_equality() {
        let native_version = NativeWSLVersion {
            Major: 1,
            Minor: 2,
            Revision: 3,
        };
        let version1 = WSLVersion::from_raw(&native_version as *const _ as *const wslplugins_sys::WSLVersion);
        let version2 = WSLVersion::from_raw(&native_version as *const _ as *const wslplugins_sys::WSLVersion);
        assert_eq!(version1.partial_cmp(&version2), Some(std::cmp::Ordering::Equal));
    }

    #[cfg(feature = "semver")]
    #[test]
    fn test_display_wslversionerror() {
        assert_eq!(format!("{}", WSLVersionError::PreReleaseNotEmpty), "Pre-release field is not empty");
        assert_eq!(format!("{}", WSLVersionError::BuildMetadataNotEmpty), "Build metadata field is not empty");
    }
}
