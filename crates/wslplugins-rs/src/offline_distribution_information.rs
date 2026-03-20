//! # Offline Distribution Information
//!
//! This module provides an abstraction over `WslOfflineDistributionInformation` from the WSL Plugin API,
//! offering a safe and idiomatic Rust interface for accessing offline distribution details.

use crate::{
    api::{
        errors::require_update_error::Result, utils::check_required_version_result_from_context,
    },
    core_distribution_information::CoreDistributionInformation,
    UserDistributionID, WSLContext, WSLVersion,
};
use std::{
    ffi::OsString,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    os::windows::ffi::OsStringExt as _,
    ptr,
};
use windows_core::PCWSTR;

/// A wrapper around `WslOfflineDistributionInformation` providing a safe interface.
///
/// This struct allows access to the details of an offline WSL distribution, including
/// its ID, name, and optional package family name.
#[repr(transparent)]
pub struct OfflineDistributionInformation(wslpluginapi_sys::WslOfflineDistributionInformation);

impl From<OfflineDistributionInformation> for wslpluginapi_sys::WslOfflineDistributionInformation {
    #[inline]
    fn from(value: OfflineDistributionInformation) -> Self {
        value.0
    }
}

impl From<wslpluginapi_sys::WslOfflineDistributionInformation> for OfflineDistributionInformation {
    #[inline]
    fn from(value: wslpluginapi_sys::WslOfflineDistributionInformation) -> Self {
        Self(value)
    }
}

impl AsRef<wslpluginapi_sys::WslOfflineDistributionInformation> for OfflineDistributionInformation {
    #[inline]
    fn as_ref(&self) -> &wslpluginapi_sys::WslOfflineDistributionInformation {
        &self.0
    }
}

impl AsRef<OfflineDistributionInformation> for wslpluginapi_sys::WslOfflineDistributionInformation {
    #[inline]
    fn as_ref(&self) -> &OfflineDistributionInformation {
        // SAFETY: This conversion is safe because of transparency.
        unsafe { &*ptr::from_ref::<Self>(self).cast::<OfflineDistributionInformation>() }
    }
}

impl CoreDistributionInformation for OfflineDistributionInformation {
    #[inline]
    fn id(&self) -> UserDistributionID {
        self.0.Id.into()
    }

    /// Retrieves the name of the offline distribution as an [`OsString`].
    #[inline]
    fn name(&self) -> OsString {
        // SAFETY: name is known to be valid
        unsafe { OsString::from_wide(PCWSTR::from_raw(self.0.Name).as_wide()) }
    }

    /// Retrieves the package family name of the offline distribution, if available.
    ///
    /// # Returns
    /// - `Some(OsString)`: If the package family name is set.
    /// - `None`: If the package family name is null or empty.
    #[inline]
    fn package_family_name(&self) -> Option<OsString> {
        // SAFETY: check already inside
        unsafe {
            let ptr = PCWSTR::from_raw(self.0.PackageFamilyName);
            if ptr.is_null() || ptr.is_empty() {
                None
            } else {
                Some(OsString::from_wide(ptr.as_wide()))
            }
        }
    }

    #[inline]
    fn flavor(&self) -> Result<Option<OsString>> {
        check_required_version_result_from_context(
            WSLContext::get_current(),
            &WSLVersion::new(2, 4, 4),
        )?;

        // SAFETY: check already inside
        unsafe {
            let ptr = PCWSTR::from_raw(self.0.Flavor);
            if ptr.is_null() || ptr.is_empty() {
                Ok(None)
            } else {
                Ok(Some(OsString::from_wide(ptr.as_wide())))
            }
        }
    }

    #[inline]
    fn version(&self) -> Result<Option<OsString>> {
        check_required_version_result_from_context(
            WSLContext::get_current(),
            &WSLVersion::new(2, 4, 4),
        )?;
        // SAFETY: check already inside
        unsafe {
            let ptr = PCWSTR::from_raw(self.0.Version);
            if ptr.is_null() || ptr.is_empty() {
                Ok(None)
            } else {
                Ok(Some(OsString::from_wide(ptr.as_wide())))
            }
        }
    }
}

impl<T> PartialEq<T> for OfflineDistributionInformation
where
    T: CoreDistributionInformation,
{
    /// Compares two distributions by their IDs for equality.
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.id() == other.id()
    }
}

impl Hash for OfflineDistributionInformation {
    /// Computes a hash based on the distribution's ID.
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Display for OfflineDistributionInformation {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: Name is known to be valid
        unsafe {
            write!(
                f,
                "{} {{{}}}",
                PCWSTR::from_raw(self.0.Name).display(),
                self.id()
            )
        }
    }
}

impl Debug for OfflineDistributionInformation {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("DistributionInformation");
        dbg.field("name", &self.name())
            .field("id", &self.id())
            .field("package_family_name", &self.package_family_name());
        let mut exhaustive = true;
        if let Ok(flavor) = self.flavor() {
            dbg.field("flavor", &flavor);
        } else {
            exhaustive = false;
        }
        if let Ok(version) = self.version() {
            dbg.field("version", &version);
        } else {
            exhaustive = false;
        }
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
        test_transparence::<
            wslpluginapi_sys::WslOfflineDistributionInformation,
            OfflineDistributionInformation,
        >();
    }
}
