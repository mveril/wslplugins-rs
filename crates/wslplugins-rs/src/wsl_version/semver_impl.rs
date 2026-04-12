use super::WSLVersion;
use semver::{BuildMetadata, Prerelease, Version};
use std::num::TryFromIntError;
use thiserror::Error;

/// Errors returned when converting a [`semver::Version`] into [`WSLVersion`].
///
/// `WSLVersion` only models the numeric `major.minor.revision` components used
/// by the WSL plugin API. Semantic-versioning pre-release identifiers and build
/// metadata therefore cannot be represented and are rejected.
#[cfg_attr(docsrs, doc(cfg(feature = "semver")))]
#[derive(Debug, Error)]
pub enum SemverConversionError {
    /// The semantic version contains a pre-release identifier such as
    /// `-alpha.1`, which has no equivalent in `WSLVersion`.
    #[error("semantic versions with pre-release identifiers cannot be converted into WSLVersion")]
    PrereleaseNotSupported,
    /// The semantic version contains build metadata such as `+build.1`, which
    /// has no equivalent in `WSLVersion`.
    #[error("semantic versions with build metadata cannot be converted into WSLVersion")]
    BuildMetadataNotSupported,
    /// One of the numeric version components does not fit into the `u32`
    /// representation used by `WSLVersion`.
    #[error("semantic version numeric component exceeds u32 range")]
    ComponentOutOfRange(#[from] TryFromIntError),
}

impl From<WSLVersion> for Version {
    #[inline]
    fn from(value: WSLVersion) -> Self {
        Self {
            major: u64::from(value.major()),
            minor: u64::from(value.minor()),
            patch: u64::from(value.revision()),
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        }
    }
}

impl TryFrom<Version> for WSLVersion {
    type Error = SemverConversionError;

    #[inline]
    fn try_from(value: Version) -> Result<Self, Self::Error> {
        if !value.pre.is_empty() {
            return Err(SemverConversionError::PrereleaseNotSupported);
        }

        if !value.build.is_empty() {
            return Err(SemverConversionError::BuildMetadataNotSupported);
        }

        Ok(Self::new(
            u32::try_from(value.major)?,
            u32::try_from(value.minor)?,
            u32::try_from(value.patch)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_try_from_semver_version_rejects_prerelease() {
        let semver_version = Version::parse("2.4.4-alpha.1").unwrap();

        let error = WSLVersion::try_from(semver_version).unwrap_err();

        assert!(matches!(
            error,
            SemverConversionError::PrereleaseNotSupported
        ));
    }

    #[test]
    fn test_try_from_semver_version_rejects_build_metadata() {
        let semver_version = Version::parse("2.4.4+build.1").unwrap();

        let error = WSLVersion::try_from(semver_version).unwrap_err();

        assert!(matches!(
            error,
            SemverConversionError::BuildMetadataNotSupported
        ));
    }

    #[test]
    fn test_try_from_semver_version_rejects_out_of_range_component() {
        let semver_version = Version::new(u64::from(u32::MAX) + 1, 0, 0);

        let error = WSLVersion::try_from(semver_version).unwrap_err();

        assert!(matches!(
            error,
            SemverConversionError::ComponentOutOfRange(_)
        ));
    }

    proptest! {
        #[test]
        fn proptest_into_semver_preserves_components(
            major in any::<u32>(),
            minor in any::<u32>(),
            revision in any::<u32>(),
        ) {
            let version = WSLVersion::new(major, minor, revision);
            let semver_version: Version = version.into();

            prop_assert_eq!(semver_version.major, u64::from(major));
            prop_assert_eq!(semver_version.minor, u64::from(minor));
            prop_assert_eq!(semver_version.patch, u64::from(revision));
            prop_assert!(semver_version.pre.is_empty());
            prop_assert!(semver_version.build.is_empty());
        }

        #[test]
        fn proptest_try_from_semver_roundtrip(
            major in any::<u32>(),
            minor in any::<u32>(),
            patch in any::<u32>(),
        ) {
            let semver_version = Version::new(
                u64::from(major),
                u64::from(minor),
                u64::from(patch),
            );

            let version = WSLVersion::try_from(semver_version).unwrap();

            prop_assert_eq!(version, WSLVersion::new(major, minor, patch));
        }
    }
}
