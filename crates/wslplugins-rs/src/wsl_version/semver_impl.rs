use super::WSLVersion;
use semver::{BuildMetadata, Prerelease, Version};
use std::num::TryFromIntError;
use thiserror::Error;

/// Errors returned when converting a [`semver::Version`] into [`WSLVersion`].
///
/// `WSLVersion` only models the numeric `major.minor.revision` components used
/// by the WSL plugin API. Semantic-versioning pre-release identifiers and build
/// metadata therefore cannot be represented and are rejected.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
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
    ComponentOutOfRange,
}

impl From<TryFromIntError> for SemverConversionError {
    #[inline]
    fn from(_: TryFromIntError) -> Self {
        Self::ComponentOutOfRange
    }
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
        let semver_version = Version {
            major: 2,
            minor: 4,
            patch: 4,
            #[allow(clippy::unwrap_used, reason = "test data is valid")]
            pre: "alpha.1".parse().unwrap(),
            build: BuildMetadata::EMPTY,
        };

        let result = WSLVersion::try_from(semver_version);

        assert_eq!(result, Err(SemverConversionError::PrereleaseNotSupported));
    }

    #[test]
    fn test_try_from_semver_version_rejects_build_metadata() {
        let semver_version = Version {
            major: 2,
            minor: 4,
            patch: 4,
            pre: Prerelease::EMPTY,
            #[allow(clippy::unwrap_used, reason = "test data is valid")]
            build: "build.1".parse().unwrap(),
        };

        let result = WSLVersion::try_from(semver_version);

        assert_eq!(
            result,
            Err(SemverConversionError::BuildMetadataNotSupported)
        );
    }

    #[test]
    fn test_try_from_semver_version_rejects_out_of_range_component() {
        let semver_version = Version::new(u64::from(u32::MAX) + 1, 0, 0);

        let result = WSLVersion::try_from(semver_version);

        assert_eq!(result, Err(SemverConversionError::ComponentOutOfRange));
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
            prop_assert_eq!(semver_version.pre, Prerelease::EMPTY);
            prop_assert_eq!(semver_version.build, BuildMetadata::EMPTY);
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

            prop_assert_eq!(
                WSLVersion::try_from(semver_version),
                Ok(WSLVersion::new(major, minor, patch))
            );
        }
    }
}
