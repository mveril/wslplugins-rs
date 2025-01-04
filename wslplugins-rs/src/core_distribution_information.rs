//! # Core Distribution Information
//!
//! This module defines a trait to represent core information about a WSL distribution.
//! It provides methods to retrieve essential details such as the distribution ID, name,
//! and package family name, offering a consistent interface for interacting with WSL distributions.
//!
//! ## Overview
//! The `CoreDistributionInformation` trait is designed to abstract the key properties
//! of a distribution. Implementing this trait allows for seamless integration with systems
//! that need to handle multiple distributions in a consistent manner.

use std::ffi::OsString;
use windows::core::GUID;

/// A trait representing the core information of a WSL distribution.
///
/// This trait abstracts the common properties of a WSL distribution, such as its unique ID,
/// display name, and package family name (if applicable).
pub trait CoreDistributionInformation {
    /// Retrieves the unique ID of the distribution.
    ///
    /// The ID is guaranteed to remain the same across reboots.
    ///
    /// # Returns
    /// A reference to the [GUID] representing the distribution's unique identifier.
    fn id(&self) -> &GUID;

    /// Retrieves the name of the distribution.
    ///
    /// # Returns
    /// An [OsString] containing the display name of the distribution.
    fn name(&self) -> OsString;

    /// Retrieves the package family name of the distribution, if available.
    ///
    /// The package family name is applicable if the distribution is packaged.
    ///
    /// # Returns
    /// - `Some`(OsString)`: If the distribution has a package family name.
    /// - `None`: If the distribution is not packaged or the information is unavailable.
    fn package_family_name(&self) -> Option<OsString>;
}
