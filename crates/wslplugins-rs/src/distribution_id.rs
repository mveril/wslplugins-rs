//! Distribution identifiers used by the crate.
//!
//! [`DistributionID`] models the two kinds of distributions exposed by WSL:
//! the shared system distribution and user-installed distributions identified by
//! a [`UserDistributionID`].
//!
//! The module also exposes the conversions commonly needed by the API surface:
//!
//! - converting from a [`UserDistributionID`] or `Option<UserDistributionID>`
//!   into a [`DistributionID`],
//! - converting a [`DistributionID`] back into `Option<UserDistributionID>`,
//! - retrieving a distribution identifier from any
//!   [`CoreWSLDistributionInformation`] implementation.
//!
//! When a caller needs a user distribution identifier and receives
//! [`DistributionID::System`] instead, [`UserDistributionIDConversionError`] is returned.

use crate::{CoreWSLDistributionInformation, UserDistributionID};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub use crate::user_distribution_id::UserDistributionIDConversionError;

/// Represents a distribution identifier in the Windows Subsystem for Linux (WSL).
///
/// A distribution can either be the system-level distribution or a user-specific distribution
/// identified by a [`UserDistributionID`].
///
/// ## Variants
///
/// - `System`: Represents the system distribution, a central distribution used by WSL
///   for managing low-level functionalities such as audio and graphical interaction.
///   Refer to the [WSLg Architecture blogpost](https://devblogs.microsoft.com/commandline/wslg-architecture/#system-distro).
///
/// - `User(UserDistributionID)`: Represents an individual distribution installed by a user. Each distribution
///   is uniquely identified by a [`UserDistributionID`], which is consistent across reboots.
///
/// ## Note
///
/// - The system distribution serves as a foundational component in WSL, often interacting with
///   user distributions for operations like Linux GUI apps.
/// - User distributions provide isolated environments for specific Linux distributions, allowing
///   users to install and run various Linux distributions on their Windows machines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DistributionID {
    /// Represents the system-level distribution.
    /// For more info about the system distribution please check the [WSLg architecture blogpost](https://devblogs.microsoft.com/commandline/wslg-architecture/#system-distro)
    System,
    /// Represents an installed user-specific distribution identified by a [`UserDistributionID`].
    User(UserDistributionID),
}

impl DistributionID {
    /// Checks if the distribution is a distribution installed by the user.
    #[must_use]
    #[inline]
    pub const fn is_user(&self) -> bool {
        matches!(*self, Self::User(_))
    }
    /// Checks if the distribution is the system distribution.
    #[must_use]
    #[inline]
    pub const fn is_system(&self) -> bool {
        matches!(*self, Self::System)
    }
}

impl From<UserDistributionID> for DistributionID {
    #[inline]
    fn from(value: UserDistributionID) -> Self {
        Self::User(value)
    }
}

impl From<&UserDistributionID> for DistributionID {
    #[inline]
    fn from(value: &UserDistributionID) -> Self {
        Self::User(*value)
    }
}

impl<T: CoreWSLDistributionInformation> From<&T> for DistributionID {
    /// Converts a reference to a type implementing `CoreWSLDistributionInformation` into a `DistributionID`.
    #[inline]
    fn from(value: &T) -> Self {
        value.id().into()
    }
}

impl From<Option<UserDistributionID>> for DistributionID {
    /// Converts an `Option<GUID>` into a `DistributionID`, defaulting to `System` if `None`.
    #[inline]
    fn from(value: Option<UserDistributionID>) -> Self {
        value.map_or(Self::System, Self::User)
    }
}

impl From<DistributionID> for Option<UserDistributionID> {
    /// Converts a `DistributionID` into an `Option<GUID>`.
    #[inline]
    fn from(value: DistributionID) -> Self {
        match value {
            DistributionID::System => None,
            DistributionID::User(id) => Some(id),
        }
    }
}

impl Display for DistributionID {
    /// Formats the `DistributionID` for display.
    ///
    /// Displays "System" for the system-level distribution, or the GUID for a user-specific distribution.
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => f.write_str("System"),
            Self::User(id) => std::fmt::Display::fmt(id, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::errors::require_update_error::Result;
    use proptest::prelude::*;
    use std::ffi::OsString;

    #[derive(Clone, Copy)]
    struct TestDistribution {
        id: UserDistributionID,
    }

    impl CoreWSLDistributionInformation for TestDistribution {
        fn id(&self) -> UserDistributionID {
            self.id
        }

        fn name(&self) -> OsString {
            OsString::from("test")
        }

        fn package_family_name(&self) -> Option<OsString> {
            None
        }

        fn flavor(&self) -> Result<Option<OsString>> {
            Ok(None)
        }

        fn version(&self) -> Result<Option<OsString>> {
            Ok(None)
        }
    }

    #[test]
    fn is_system_returns_true_for_system_distribution() {
        assert!(DistributionID::System.is_system());
    }

    #[test]
    fn is_user_returns_false_for_system_distribution() {
        assert!(!DistributionID::System.is_user());
    }

    #[test]
    fn from_none_returns_system() {
        assert_eq!(
            DistributionID::from(Option::<UserDistributionID>::None),
            DistributionID::System
        );
    }

    #[test]
    fn option_from_system_returns_none() {
        let maybe_user: Option<UserDistributionID> = DistributionID::System.into();
        assert_eq!(maybe_user, None);
    }

    #[test]
    fn display_for_system_is_system() {
        assert_eq!(format!("{}", DistributionID::System), "System");
    }

    proptest! {
        #[test]
        fn is_system_returns_false_for_user_distribution(raw_id in any::<u128>()) {
            let user_id = UserDistributionID::from(windows_core::GUID::from_u128(raw_id));

            prop_assert!(!DistributionID::User(user_id).is_system());
        }

        #[test]
        fn is_user_returns_true_for_user_distribution(raw_id in any::<u128>()) {
            let user_id = UserDistributionID::from(windows_core::GUID::from_u128(raw_id));

            prop_assert!(DistributionID::User(user_id).is_user());
        }

        #[test]
        fn try_from_user_returns_inner_id(raw_id in any::<u128>()) {
            let user_id = UserDistributionID::from(windows_core::GUID::from_u128(raw_id));

            prop_assert_eq!(
                UserDistributionID::try_from(DistributionID::User(user_id)),
                Ok(user_id)
            );
        }

        #[test]
        fn from_user_distribution_id_wraps_user_variant(raw_id in any::<u128>()) {
            let user_id = UserDistributionID::from(windows_core::GUID::from_u128(raw_id));

            prop_assert_eq!(DistributionID::from(user_id), DistributionID::User(user_id));
        }

        #[test]
        fn from_user_distribution_id_reference_wraps_user_variant(raw_id in any::<u128>()) {
            let user_id = UserDistributionID::from(windows_core::GUID::from_u128(raw_id));

            prop_assert_eq!(DistributionID::from(&user_id), DistributionID::User(user_id));
        }

        #[test]
        fn from_some_user_distribution_id_wraps_user_variant(raw_id in any::<u128>()) {
            let user_id = UserDistributionID::from(windows_core::GUID::from_u128(raw_id));

            prop_assert_eq!(
                DistributionID::from(Some(user_id)),
                DistributionID::User(user_id)
            );
        }

        #[test]
        fn option_from_user_distribution_returns_some_inner_id(raw_id in any::<u128>()) {
            let user_id = UserDistributionID::from(windows_core::GUID::from_u128(raw_id));
            let maybe_user: Option<UserDistributionID> = DistributionID::User(user_id).into();

            prop_assert_eq!(maybe_user, Some(user_id));
        }

        #[test]
        fn display_for_user_delegates_to_user_distribution_id(raw_id in any::<u128>()) {
            let user_id = UserDistributionID::from(windows_core::GUID::from_u128(raw_id));

            prop_assert_eq!(
                format!("{}", DistributionID::User(user_id)),
                format!("{user_id}")
            );
        }

        #[test]
        fn from_core_distribution_information_uses_the_underlying_id(raw_id in any::<u128>()) {
            let user_id = UserDistributionID::from(windows_core::GUID::from_u128(raw_id));
            let distribution = TestDistribution { id: user_id };

            prop_assert_eq!(DistributionID::from(&distribution), DistributionID::User(user_id));
        }
    }
}
