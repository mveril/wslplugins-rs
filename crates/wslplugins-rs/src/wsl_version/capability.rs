use super::WSLVersion;
use crate::api::errors::RequirementDefinition;
use std::collections::HashSet;
use strum::{Display, EnumIter};

/// WSL Plugin API capabilities that are only available from a specific API version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter)]
#[non_exhaustive]
pub enum WSLVersionCapability {
    /// Access to `WSLDistributionInformation::InitPid` (`2.0.5`).
    DistributionInitPid,
    /// Notification sent when a distribution is registered (`2.1.2`).
    DistributionRegisteredHook,
    /// Notification sent when a distribution is unregistered (`2.1.2`).
    DistributionUnregisteredHook,
    /// Execute a command inside a specific user distribution (`2.1.2`).
    ExecuteBinaryInDistribution,
    /// Access to the distribution flavor field (`2.4.4`).
    DistributionFlavor,
    /// Access to the distribution version field (`2.4.4`).
    DistributionVersion,
}

impl WSLVersionCapability {
    /// Returns the minimum WSL Plugin API version required for this capability.
    #[must_use]
    #[inline]
    pub const fn required_version(self) -> WSLVersion {
        match self {
            Self::DistributionInitPid => WSLVersion::new(2, 0, 5),
            Self::DistributionRegisteredHook
            | Self::DistributionUnregisteredHook
            | Self::ExecuteBinaryInDistribution => WSLVersion::new(2, 1, 2),
            Self::DistributionFlavor | Self::DistributionVersion => WSLVersion::new(2, 4, 4),
        }
    }
}

impl TryFrom<RequirementDefinition> for WSLVersionCapability {
    type Error = RequirementDefinition;

    #[inline]
    fn try_from(value: RequirementDefinition) -> std::result::Result<Self, Self::Error> {
        match value {
            RequirementDefinition::Capabilities(capabilities) if capabilities.len() == 1 => {
                capabilities
                    .into_iter()
                    .next()
                    .ok_or_else(|| RequirementDefinition::Capabilities(HashSet::new()))
            }
            requirement => Err(requirement),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use strum::IntoEnumIterator as _;

    fn arb_capability() -> impl Strategy<Value = WSLVersionCapability> {
        prop::sample::select(WSLVersionCapability::iter().collect::<Vec<_>>())
    }

    #[test]
    fn single_capability_requirement_can_be_recovered() {
        let capability = WSLVersionCapability::ExecuteBinaryInDistribution;
        let requirement = RequirementDefinition::from(capability);

        assert_eq!(WSLVersionCapability::try_from(requirement), Ok(capability));
    }

    #[test]
    fn multi_capability_requirement_cannot_be_recovered_as_single_capability() {
        let requirement = RequirementDefinition::from([
            WSLVersionCapability::DistributionRegisteredHook,
            WSLVersionCapability::DistributionUnregisteredHook,
        ]);

        assert!(WSLVersionCapability::try_from(requirement).is_err());
    }

    proptest! {
        #[test]
        fn single_capability_requirement_roundtrips(capability in arb_capability()) {
            let requirement = RequirementDefinition::from(capability);

            prop_assert_eq!(WSLVersionCapability::try_from(requirement), Ok(capability));
        }
    }
}
