//! # Module `DistributionID`
//!
//! This module defines an abstraction to represent WSL distributions through a
//! [`DistributionID`]. It supports two types of identifiers: system-level distributions
//! and user-specific installed distributions identified by a GUID.
//!
//! ## Key Features
//!
//! - Bi-directional conversion between [`DistributionID`] and [GUID].
//! - Robust error handling for conversions via [`ConversionError`].
//! - Display implementation ([Display]) and support for other idiomatic conversions.
//!
//! ## Usage Context
//!
//! This abstraction is particularly useful in environments where WSL requires
//! distribution identification via GUIDs or when a distinction between a system-level
//! distribution and a user-specific distribution is necessary. The associated functions
//! and conversions simplify integration with APIs like those defined in `WslPluginApi`.
use crate::CoreDistributionInformation;
use std::{convert::TryFrom, fmt::Display};
use thiserror::Error;
use windows_core::GUID;

/// Represents a distribution identifier in WSL.
///
/// This can either be the system-level distribution or a user-specific distribution
/// identified by a GUID.
#[derive(Debug, Clone, Copy)]
pub enum DistributionID {
    /// Represents the system-level distribution.
    System,
    /// Represents an installed user-specific distribution identified by a [GUID].
    User(GUID),
}

/// Error type for conversion failures between `DistributionID` and GUID.
#[derive(Debug, Error)]
#[error("Cannot convert System distribution to GUID.")]
pub struct ConversionError;

impl TryFrom<DistributionID> for GUID {
    type Error = ConversionError;

    /// Attempts to convert a `DistributionID` into a GUID.
    ///
    /// # Errors
    /// Returns `ConversionError` if the `DistributionID` is `System`.
    #[inline]
    fn try_from(value: DistributionID) -> Result<Self, Self::Error> {
        match value {
            DistributionID::User(id) => Ok(id),
            DistributionID::System => Err(ConversionError),
        }
    }
}

impl From<GUID> for DistributionID {
    /// Converts a GUID into a `DistributionID`.
    #[inline]
    fn from(value: GUID) -> Self {
        Self::User(value)
    }
}

impl<T: CoreDistributionInformation> From<T> for DistributionID {
    /// Converts a type implementing `CoreDistributionInformation` into a `DistributionID`.
    #[inline]
    fn from(value: T) -> Self {
        windows_core::GUID::from(value.id()).into()
    }
}

impl From<Option<GUID>> for DistributionID {
    /// Converts an `Option<GUID>` into a `DistributionID`, defaulting to `System` if `None`.
    #[inline]
    fn from(value: Option<GUID>) -> Self {
        value.map_or(Self::System, Self::User)
    }
}

impl From<DistributionID> for Option<GUID> {
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
            Self::User(id) => std::fmt::Debug::fmt(id, f),
        }
    }
}
