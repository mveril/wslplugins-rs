//! # Module `DistributionID`
//!
//! This module defines an abstraction to represent WSL distributions through a
//! [`DistributionID`]. It supports two types of identifiers: system-level distributions
//! and user-specific installed distributions identified by a GUID.
//!
//! ## Key Features
//!
//! - Bi-directional conversion between [`DistributionID`] and [`UserDistributionID`].
//! - Robust error handling for conversions via [`ConversionError`].
//! - Display implementation ([Display]) and support for other idiomatic conversions.
//!
//! ## Usage Context
//!
//! This abstraction is particularly useful in environments where WSL requires
//! distribution identification via GUIDs or when a distinction between a system-level
//! distribution and a user-specific distribution is necessary. The associated functions
//! and conversions simplify integration with APIs like those defined in `WslPluginApi`.

use crate::{CoreDistributionInformation, UserDistributionID};
use std::{convert::TryFrom, fmt::Display};
use thiserror::Error;

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
#[derive(Debug, Clone, Copy)]
pub enum DistributionID {
    /// Represents the system-level distribution.
    /// For more info about the system distribution please check the [WSLg architecture blogpost](https://devblogs.microsoft.com/commandline/wslg-architecture/#system-distro)
    System,
    /// Represents an installed user-specific distribution identified by a [`UserDistributionID`].
    User(UserDistributionID),
}

/// Error type for conversion failures between [`DistributionID`] and [`UserDistributionID`].
#[derive(Debug, Error)]
#[error("Cannot convert System distribution to UserDistribution.")]
pub struct ConversionError;

impl TryFrom<DistributionID> for UserDistributionID {
    type Error = ConversionError;
    #[inline]
    fn try_from(value: DistributionID) -> Result<Self, Self::Error> {
        match value {
            DistributionID::User(id) => Ok(id),
            DistributionID::System => Err(ConversionError),
        }
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

impl<T: CoreDistributionInformation> From<&T> for DistributionID {
    /// Converts a reference to a type implementing `CoreDistributionInformation` into a `DistributionID`.
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
