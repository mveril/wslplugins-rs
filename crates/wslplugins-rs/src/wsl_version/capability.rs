use super::WSLVersion;
use strum::EnumIter;

/// WSL Plugin API capabilities that are only available from a specific API version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
#[non_exhaustive]
pub enum WSLVersionCapability {
    /// Access to `WSLDistributionInformation::InitPid`.
    DistributionInitPid,
    /// Notification sent when a distribution is registered.
    DistributionRegisteredHook,
    /// Notification sent when a distribution is unregistered.
    DistributionUnregisteredHook,
    /// Execute a command inside a specific user distribution.
    ExecuteBinaryInDistribution,
    /// Access to the distribution flavor field.
    DistributionFlavor,
    /// Access to the distribution version field.
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
