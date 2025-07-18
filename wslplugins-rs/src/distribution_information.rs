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

#[cfg(doc)]
use crate::api::errors::require_update_error::Error;
use crate::api::{
    errors::require_update_error::Result, utils::check_required_version_result_from_context,
};
use crate::core_distribution_information::CoreDistributionInformation;
use crate::WSLContext;
use crate::WSLVersion;
use std::ffi::OsString;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use windows::core::{GUID, PCWSTR};

/// Represents detailed information about a WSL distribution.
///
/// This struct wraps the `WSLDistributionInformation` from the WSL Plugin API and provides
/// safe, idiomatic Rust access to its fields.
#[repr(transparent)]
pub struct DistributionInformation(wslpluginapi_sys::WSLDistributionInformation);

impl AsRef<DistributionInformation> for wslpluginapi_sys::WSLDistributionInformation {
    fn as_ref(&self) -> &DistributionInformation {
        unsafe {
            &*(self as *const wslpluginapi_sys::WSLDistributionInformation
                as *const DistributionInformation)
        }
    }
}

impl From<DistributionInformation> for wslpluginapi_sys::WSLDistributionInformation {
    fn from(value: DistributionInformation) -> Self {
        value.0
    }
}

impl AsRef<wslpluginapi_sys::WSLDistributionInformation> for DistributionInformation {
    fn as_ref(&self) -> &wslpluginapi_sys::WSLDistributionInformation {
        &self.0
    }
}

impl From<wslpluginapi_sys::WSLDistributionInformation> for DistributionInformation {
    fn from(value: wslpluginapi_sys::WSLDistributionInformation) -> Self {
        DistributionInformation(value)
    }
}

impl DistributionInformation {
    /// Retrieves the PID of the init process.
    ///
    /// This requires API version 2.0.5 or higher. If the current API version does not meet
    /// the requirement, an error is returned.
    ///
    /// # Returns
    /// - `Ok(pid)`: The PID of the init process.
    /// # Errors
    /// [Error]: If the runtime version version is insufficient.
    pub fn init_pid(&self) -> Result<u32> {
        check_required_version_result_from_context(
            WSLContext::get_current(),
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

impl CoreDistributionInformation for DistributionInformation {
    fn id(&self) -> GUID {
        unsafe { mem::transmute_copy(&self.0.Id) }
    }

    fn name(&self) -> OsString {
        unsafe { OsString::from_wide(PCWSTR::from_raw(self.0.Name).as_wide()) }
    }

    fn package_family_name(&self) -> Option<OsString> {
        unsafe {
            let ptr = self.0.PackageFamilyName;
            if ptr.is_null() {
                None
            } else {
                Some(OsString::from_wide(PCWSTR::from_raw(ptr).as_wide()))
            }
        }
    }

    fn flavor(&self) -> Result<Option<OsString>> {
        check_required_version_result_from_context(
            WSLContext::get_current(),
            &WSLVersion::new(2, 4, 4),
        )?;
        unsafe {
            let ptr = self.0.Flavor;
            if ptr.is_null() {
                Ok(None)
            } else {
                Ok(Some(OsString::from_wide(PCWSTR::from_raw(ptr).as_wide())))
            }
        }
    }

    fn version(&self) -> Result<Option<OsString>> {
        check_required_version_result_from_context(
            WSLContext::get_current(),
            &WSLVersion::new(2, 4, 4),
        )?;
        unsafe {
            let ptr = self.0.Flavor;
            if ptr.is_null() {
                Ok(None)
            } else {
                Ok(Some(OsString::from_wide(PCWSTR::from_raw(ptr).as_wide())))
            }
        }
    }
}

impl<T> PartialEq<T> for DistributionInformation
where
    T: CoreDistributionInformation,
{
    /// Compares two distributions for equality based on their IDs.
    fn eq(&self, other: &T) -> bool {
        self.id() == other.id()
    }
}

impl Hash for DistributionInformation {
    /// Computes a hash based on the distribution's ID.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Display for DistributionInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                "{:} {{{:?}}}",
                PCWSTR::from_raw(self.0.Name).display(),
                self.id()
            )
        }
    }
}

impl Debug for DistributionInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("DistributionInformation");
        dbg.field("name", &self.name())
            .field("id", &self.id())
            .field("package_family_name", &self.package_family_name())
            .field("pid_namespace", &self.pid_namespace());
        let mut exhaustive = true;

        if let Ok(pid) = self.init_pid() {
            dbg.field("init_pid", &pid);
        } else {
            exhaustive = false;
        };
        if let Ok(flavor) = self.flavor() {
            dbg.field("flavor", &flavor);
        } else {
            exhaustive = false;
        };
        if let Ok(version) = self.version() {
            dbg.field("version", &version);
        } else {
            exhaustive = false;
        };

        if exhaustive {
            dbg.finish()
        } else {
            dbg.finish_non_exhaustive()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_transparence;

    #[test]
    fn test_layouts() {
        test_transparence::<wslpluginapi_sys::WSLDistributionInformation, DistributionInformation>(
        );
    }
}
