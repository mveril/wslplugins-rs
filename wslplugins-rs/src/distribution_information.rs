//! # Distribution Information
//!
//! This module provides a safe abstraction for accessing information about a WSL distribution.
//! It wraps the `WSLDistributionInformation` structure from the WSL Plugin API and implements
//! the `CoreDistributionInformation` trait for consistent access to distribution details.
//!
//! ## Overview
//! The `DistributionInformation` struct provides methods to retrieve:
//! - Distribution ID
//! - Distribution name
//! - Package family name (if applicable)
//! - Process ID (PID) of the init process (requires API version 2.0.5 or higher)
//! - PID namespace

extern crate wslplugins_sys;
#[cfg(doc)]
use crate::api::errors::require_update_error::Error;
use crate::api::{
    errors::require_update_error::Result, utils::check_required_version_result_from_context,
};
use crate::core_distribution_information::CoreDistributionInformation;
use crate::WSLContext;
use std::ffi::OsString;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::os::windows::ffi::OsStringExt;
use windows::core::GUID;
use wslplugins_sys::WSLVersion;

/// Represents detailed information about a WSL distribution.
///
/// This struct wraps the `WSLDistributionInformation` from the WSL Plugin API and provides
/// safe, idiomatic Rust access to its fields.
pub struct DistributionInformation<'a>(&'a wslplugins_sys::WSLDistributionInformation);

impl<'a> From<&'a wslplugins_sys::WSLDistributionInformation> for DistributionInformation<'a> {
    /// Creates a `DistributionInformation` instance from a reference to the raw WSL Plugin API structure.
    ///
    /// # Arguments
    /// - `ptr`: A reference to a `WSLDistributionInformation` instance.
    ///
    /// # Returns
    /// A wrapped `DistributionInformation` instance.
    fn from(ptr: &'a wslplugins_sys::WSLDistributionInformation) -> Self {
        Self(ptr)
    }
}

impl DistributionInformation<'_> {
    /// Retrieves the PID of the init process.
    ///
    /// This requires API version 2.0.5 or higher. If the current API version does not meet
    /// the requirement, an error is returned.
    ///
    /// # Returns
    /// - `Ok(u32)`: The PID of the init process.
    /// # Errors
    /// [Error]: If the API version is insufficient.
    pub fn init_pid(&self) -> Result<u32> {
        check_required_version_result_from_context(
            WSLContext::get_current_or_panic(),
            &WSLVersion::new(2, 0, 5),
        )?;
        Ok(self.0.InitPid)
    }

    /// Retrieves the PID namespace for the distribution.
    ///
    /// # Returns
    /// The PID namespace as a `u64`.
    ///
    pub fn pid_namespace(&self) -> u64 {
        self.0.PidNamespace
    }
}

impl CoreDistributionInformation for DistributionInformation<'_> {
    /// Retrieves the unique ID of the distribution.
    ///
    /// # Returns
    /// A reference to the [GUID] representing the distribution's unique identifier.
    fn id(&self) -> GUID {
        self.0.Id
    }

    /// Retrieves the name of the distribution.
    ///
    /// # Returns
    /// An [OsString] containing the display name of the distribution.
    fn name(&self) -> OsString {
        unsafe { OsString::from_wide(self.0.Name.as_wide()) }
    }

    /// Retrieves the package family name of the distribution, if available.
    ///
    /// # Returns
    /// - `Some(OsString)`: If the distribution has a package family name.
    /// - `None`: If the distribution is not packaged or the information is unavailable.
    fn package_family_name(&self) -> Option<OsString> {
        unsafe {
            let ptr = self.0.PackageFamilyName;
            if ptr.is_null() {
                None
            } else {
                Some(OsString::from_wide(ptr.as_wide()))
            }
        }
    }
}

impl<T> PartialEq<T> for DistributionInformation<'_>
where
    T: CoreDistributionInformation,
{
    /// Compares two distributions for equality based on their IDs.
    fn eq(&self, other: &T) -> bool {
        self.id() == other.id()
    }
}

impl Hash for DistributionInformation<'_> {
    /// Computes a hash based on the distribution's ID.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Display for DistributionInformation<'_> {
    /// Formats the distribution information for display.
    ///
    /// The output includes the distribution's name and ID.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "{:} {{{:?}}}", self.0.Name.display(), self.0.Id) }
    }
}

impl Debug for DistributionInformation<'_> {
    /// Formats the distribution information for debugging.
    ///
    /// The output includes:
    /// - Name
    /// - ID
    /// - Package family name (if available)
    /// - PID namespace
    /// - Init PID (if available and the API version supports it)
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("DistributionInformation");
        dbg.field("name", &self.name())
            .field("id", &self.id())
            .field("package_family_name", &self.package_family_name())
            .field("pid_namespace", &self.pid_namespace());
        if let Ok(pid) = self.init_pid() {
            dbg.field("init_pid", &pid).finish()
        } else {
            dbg.finish_non_exhaustive()
        }
    }
}
