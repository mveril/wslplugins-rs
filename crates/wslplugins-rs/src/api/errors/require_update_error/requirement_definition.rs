use crate::{WSLVersion, WSLVersionCapability};
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};
use strum::IntoEnumIterator as _;

/// Defines what WSL Plugin API support is required.
///
/// A requirement can be expressed either as an explicit API version or as one
/// or more named capabilities. Capability requirements are resolved to the
/// highest minimum version required by the listed capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequirementDefinition {
    /// Requires an explicit WSL Plugin API version.
    Version(WSLVersion),
    /// Requires one or more WSL Plugin API capabilities.
    Capabilities(HashSet<WSLVersionCapability>),
}

impl RequirementDefinition {
    /// Returns the minimum WSL Plugin API version required by this definition.
    #[must_use]
    #[inline]
    pub fn version(&self) -> WSLVersion {
        match self {
            Self::Version(version) => *version,
            Self::Capabilities(capabilities) => capabilities
                .iter()
                .copied()
                .map(WSLVersionCapability::required_version)
                .max()
                .unwrap_or(WSLVersion::new(0, 0, 0)),
        }
    }
}

impl From<WSLVersion> for RequirementDefinition {
    #[inline]
    fn from(value: WSLVersion) -> Self {
        Self::Version(value)
    }
}

impl From<WSLVersionCapability> for RequirementDefinition {
    #[inline]
    fn from(value: WSLVersionCapability) -> Self {
        Self::Capabilities(HashSet::from([value]))
    }
}

impl From<HashSet<WSLVersionCapability>> for RequirementDefinition {
    #[inline]
    fn from(value: HashSet<WSLVersionCapability>) -> Self {
        Self::Capabilities(value)
    }
}

impl<const N: usize> From<[WSLVersionCapability; N]> for RequirementDefinition {
    #[inline]
    fn from(value: [WSLVersionCapability; N]) -> Self {
        Self::Capabilities(HashSet::from(value))
    }
}

impl FromIterator<WSLVersionCapability> for RequirementDefinition {
    #[inline]
    fn from_iter<T: IntoIterator<Item = WSLVersionCapability>>(iter: T) -> Self {
        Self::Capabilities(iter.into_iter().collect())
    }
}

impl Hash for RequirementDefinition {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Version(version) => {
                0_u8.hash(state);
                version.hash(state);
            }
            Self::Capabilities(capabilities) => {
                1_u8.hash(state);
                for capability in WSLVersionCapability::iter()
                    .filter(|capability| capabilities.contains(capability))
                {
                    capability.hash(state);
                }
            }
        }
    }
}

impl Display for RequirementDefinition {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Version(version) => write!(f, "version {version}"),
            Self::Capabilities(capabilities) => {
                write!(f, "capabilities ")?;
                for (index, capability) in WSLVersionCapability::iter()
                    .filter(|capability| capabilities.contains(capability))
                    .enumerate()
                {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{capability}")?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::sample::select;

    fn arb_wsl_version() -> impl Strategy<Value = WSLVersion> {
        (any::<u32>(), any::<u32>(), any::<u32>())
            .prop_map(|(major, minor, revision)| WSLVersion::new(major, minor, revision))
    }

    fn arb_capability() -> impl Strategy<Value = WSLVersionCapability> {
        select(WSLVersionCapability::iter().collect::<Vec<_>>())
    }

    #[test]
    fn version_requirement_returns_explicit_version() {
        let version = WSLVersion::new(2, 1, 2);
        let requirement = RequirementDefinition::from(version);

        assert_eq!(requirement.version(), version);
    }

    #[test]
    fn capability_requirement_returns_highest_required_version() {
        let requirement = RequirementDefinition::from([
            WSLVersionCapability::DistributionInitPid,
            WSLVersionCapability::DistributionVersion,
            WSLVersionCapability::DistributionVersion,
        ]);

        assert_eq!(requirement.version(), WSLVersion::new(2, 4, 4));
        assert_eq!(
            requirement,
            RequirementDefinition::Capabilities(HashSet::from([
                WSLVersionCapability::DistributionInitPid,
                WSLVersionCapability::DistributionVersion,
            ]))
        );
    }

    proptest! {
        #[test]
        fn version_requirement_always_returns_explicit_version(version in arb_wsl_version()) {
            let requirement = RequirementDefinition::from(version);

            prop_assert_eq!(requirement.version(), version);
        }

        #[test]
        fn capability_requirement_returns_maximum_required_version(
            capabilities in proptest::collection::hash_set(arb_capability(), 0..=6),
        ) {
            let expected = capabilities
                .iter()
                .copied()
                .map(WSLVersionCapability::required_version)
                .max()
                .unwrap_or(WSLVersion::new(0, 0, 0));
            let requirement = RequirementDefinition::from(capabilities);

            prop_assert_eq!(requirement.version(), expected);
        }
    }
}
