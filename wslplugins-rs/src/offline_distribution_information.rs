//! # Offline Distribution Information
//!
//! This module provides an abstraction over `WslOfflineDistributionInformation` from the WSL Plugin API,
//! offering a safe and idiomatic Rust interface for accessing offline distribution details.

extern crate wslplugins_sys;
use crate::core_distribution_information::CoreDistributionInformation;
use std::{
    ffi::OsString,
    fmt::{Debug, Display},
    hash::Hash,
    os::windows::ffi::OsStringExt,
};
use windows::core::GUID;

/// A wrapper around `WslOfflineDistributionInformation` providing a safe interface.
///
/// This struct allows access to the details of an offline WSL distribution, including
/// its ID, name, and optional package family name.
///
/// # Lifetime Parameters
/// - `'a`: The lifetime of the referenced `WslOfflineDistributionInformation` instance.
pub struct OfflineDistributionInformation<'a>(
    &'a wslplugins_sys::WslOfflineDistributionInformation,
);

impl<'a> OfflineDistributionInformation<'a> {
    /// Creates a new `OfflineDistributionInformation` instance from a raw pointer.
    ///
    /// # Arguments
    /// - `ptr`: A reference to a `WslOfflineDistributionInformation` instance.
    ///
    /// # Returns
    /// A safe wrapper around the provided pointer.
    pub fn from(ptr: &'a wslplugins_sys::WslOfflineDistributionInformation) -> Self {
        Self(ptr)
    }
}

impl CoreDistributionInformation for OfflineDistributionInformation<'_> {
    /// Retrieves the [GUID] of the offline distribution.
    fn id(&self) -> GUID {
        self.0.Id
    }

    /// Retrieves the name of the offline distribution as an [OsString].
    fn name(&self) -> OsString {
        unsafe { OsString::from_wide(self.0.Name.as_wide()) }
    }

    /// Retrieves the package family name of the offline distribution, if available.
    ///
    /// # Returns
    /// - `Some(OsString)`: If the package family name is set.
    /// - `None`: If the package family name is null or empty.
    fn package_family_name(&self) -> Option<OsString> {
        unsafe {
            let ptr = self.0.PackageFamilyName;
            if ptr.is_null() || ptr.is_empty() {
                None
            } else {
                Some(OsString::from_wide(ptr.as_wide()))
            }
        }
    }
}

impl<T> PartialEq<T> for OfflineDistributionInformation<'_>
where
    T: CoreDistributionInformation,
{
    /// Compares two distributions by their IDs for equality.
    fn eq(&self, other: &T) -> bool {
        self.id() == other.id()
    }
}

impl Hash for OfflineDistributionInformation<'_> {
    /// Computes a hash based on the distribution's ID.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Display for OfflineDistributionInformation<'_> {
    /// Formats the offline distribution information for display.
    ///
    /// The output includes the distribution's name and ID.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "{:} {{{:?}}}", self.0.Name.display(), self.0.Id) }
    }
}

impl Debug for OfflineDistributionInformation<'_> {
    /// Formats the offline distribution information for debugging.
    ///
    /// The output includes the distribution's name, ID, and package family name.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DistributionInformation")
            .field("name", &self.name())
            .field("id", &self.id())
            .field("package_family_name", &self.package_family_name())
            .finish()
    }
}
