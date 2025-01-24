//! # Offline Distribution Information
//!
//! This module provides an abstraction over `WslOfflineDistributionInformation` from the WSL Plugin API,
//! offering a safe and idiomatic Rust interface for accessing offline distribution details.

extern crate wslpluginapi_sys;
use crate::core_distribution_information::CoreDistributionInformation;
use std::{
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    hash::Hash,
    mem::ManuallyDrop,
    os::windows::ffi::OsStringExt,
};
use widestring::U16CString;
use windows::core::{GUID, PCWSTR};

/// A wrapper around `WslOfflineDistributionInformation` providing a safe interface.
///
/// This struct allows access to the details of an offline WSL distribution, including
/// its ID, name, and optional package family name.
#[repr(transparent)]
pub struct OfflineDistributionInformation(wslpluginapi_sys::WslOfflineDistributionInformation);

impl OfflineDistributionInformation {
    pub fn new<S: AsRef<OsStr>>(id: GUID, name: S, package_family_name: Option<S>) -> Self {
        let name_u16 = ManuallyDrop::new(U16CString::from_os_str_truncate(name.as_ref()));
        let package_familly_name = package_family_name
            .map(|pfn| ManuallyDrop::new(U16CString::from_os_str_truncate((pfn))));
        let pfn_ptr = match package_familly_name {
            Some(pfn) => PCWSTR::from_raw(pfn.as_ptr()),
            None => PCWSTR::null(),
        };
        Self(wslpluginapi_sys::WslOfflineDistributionInformation {
            Id: id,
            Name: PCWSTR::from_raw(name_u16.as_ptr()),
            PackageFamilyName: pfn_ptr,
        })
    }
}

impl From<OfflineDistributionInformation> for wslpluginapi_sys::WslOfflineDistributionInformation {
    fn from(value: OfflineDistributionInformation) -> Self {
        let value = ManuallyDrop::new(value);
        unsafe { std::ptr::read(&value.0) }
    }
}

impl From<wslpluginapi_sys::WslOfflineDistributionInformation> for OfflineDistributionInformation {
    fn from(value: wslpluginapi_sys::WslOfflineDistributionInformation) -> Self {
        OfflineDistributionInformation(value)
    }
}

impl AsRef<wslpluginapi_sys::WslOfflineDistributionInformation> for OfflineDistributionInformation {
    fn as_ref(&self) -> &wslpluginapi_sys::WslOfflineDistributionInformation {
        &self.0
    }
}

impl AsRef<OfflineDistributionInformation> for wslpluginapi_sys::WslOfflineDistributionInformation {
    fn as_ref(&self) -> &OfflineDistributionInformation {
        unsafe {
            &*(self as *const wslpluginapi_sys::WslOfflineDistributionInformation
                as *const OfflineDistributionInformation)
        }
    }
}

impl CoreDistributionInformation for OfflineDistributionInformation {
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

impl<T> PartialEq<T> for OfflineDistributionInformation
where
    T: CoreDistributionInformation,
{
    /// Compares two distributions by their IDs for equality.
    fn eq(&self, other: &T) -> bool {
        self.id() == other.id()
    }
}

impl Hash for OfflineDistributionInformation {
    /// Computes a hash based on the distribution's ID.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Display for OfflineDistributionInformation {
    /// Formats the offline distribution information for display.
    ///
    /// The output includes the distribution's name and ID.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "{:} {{{:?}}}", self.0.Name.display(), self.0.Id) }
    }
}

impl Debug for OfflineDistributionInformation {
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

impl Drop for OfflineDistributionInformation {
    fn drop(&mut self) {
        unsafe {
            widestring::U16CString::from_raw(self.0.Name.as_ptr() as *mut _);
        }
        if !self.0.PackageFamilyName.is_null() {
            unsafe {
                widestring::U16CString::from_raw(self.0.PackageFamilyName.as_ptr() as *mut _);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_transparence;

    #[test]
    fn test_layouts() {
        test_transparence::<
            wslpluginapi_sys::WslOfflineDistributionInformation,
            OfflineDistributionInformation,
        >();
    }
}
