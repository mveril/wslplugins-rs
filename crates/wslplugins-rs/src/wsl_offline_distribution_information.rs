//! # WSL Offline Distribution Information
//!
//! This module provides an abstraction over `WslOfflineDistributionInformation` from the WSL Plugin API,
//! offering a safe and idiomatic Rust interface for accessing offline distribution details.

use crate::{
    api::{errors::require_update_error::Result, utils::check_capability_result_from_context},
    core_wsl_distribution_information::CoreWSLDistributionInformation,
    utils::{opt_wide_str, wide_str},
    UserDistributionID, WSLContext, WSLVersionCapability,
};
use std::{
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
};
use widestring::U16CStr;

/// A wrapper around `WslOfflineDistributionInformation` providing a safe interface.
///
/// This struct allows access to the details of an offline WSL distribution, including
/// its ID, name, and optional package family name.
// Only fields supported by the host version are accessed through this view.
#[repr(C)]
pub struct WSLOfflineDistributionInformation {
    _private: (),
    _marker: std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
}

impl WSLOfflineDistributionInformation {
    /// Borrows a versioned WSL structure without referencing its full current layout.
    ///
    /// # Safety
    /// `raw` must be non-null, correctly aligned, and point to an immutable WSL
    /// structure containing every field supported by the current host version.
    /// The allocation and its pointees must remain valid for `'a`, which must not
    /// outlive the callback. An initialized context must describe the same host
    /// version before accessing versioned fields.
    #[inline]
    #[must_use]
    pub const unsafe fn from_raw<'a>(
        raw: *const wslpluginapi_sys::WslOfflineDistributionInformation,
    ) -> &'a Self {
        // SAFETY: The caller guarantees the allocation for this opaque borrowed view.
        unsafe { &*raw.cast::<Self>() }
    }

    /// Returns the pointer used for field-by-field ABI access.
    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const wslpluginapi_sys::WslOfflineDistributionInformation {
        std::ptr::from_ref(self).cast()
    }
}

impl CoreWSLDistributionInformation for WSLOfflineDistributionInformation {
    #[inline]
    fn id(&self) -> UserDistributionID {
        // SAFETY: This field belongs to the baseline ABI guaranteed by from_raw.
        unsafe { std::ptr::addr_of!((*self.as_ptr()).Id).read_unaligned() }.into()
    }

    /// Retrieves the name of the offline distribution as a borrowed UTF-16 string.
    #[inline]
    fn name(&self) -> &U16CStr {
        // SAFETY: The WSL Plugin API guarantees that Name points to a valid null-terminated
        // UTF-16 string for the lifetime of this distribution information.
        unsafe { wide_str(std::ptr::addr_of!((*self.as_ptr()).Name).read_unaligned()) }
    }

    /// Retrieves the package family name of the offline distribution, if available.
    ///
    /// # Returns
    /// - `Some(&U16CStr)`: If the package family name is set.
    /// - `None`: If the package family name is null or empty.
    #[inline]
    fn package_family_name(&self) -> Option<&U16CStr> {
        // SAFETY: The WSL Plugin API guarantees that a non-null PackageFamilyName remains valid
        // for the lifetime of this distribution information.
        unsafe {
            opt_wide_str(std::ptr::addr_of!((*self.as_ptr()).PackageFamilyName).read_unaligned())
        }
    }

    /// Retrieves the distribution flavor.
    ///
    /// This requires [`WSLVersionCapability::DistributionFlavor`] (`2.4.4`).
    #[inline]
    fn flavor(&self) -> Result<Option<&U16CStr>> {
        check_capability_result_from_context(
            WSLContext::get_current(),
            WSLVersionCapability::DistributionFlavor,
        )?;
        // SAFETY: The WSL Plugin API guarantees that a non-null Flavor remains valid for the
        // lifetime of this distribution information.
        Ok(unsafe { opt_wide_str(std::ptr::addr_of!((*self.as_ptr()).Flavor).read_unaligned()) })
    }

    /// Retrieves the distribution version.
    ///
    /// This requires [`WSLVersionCapability::DistributionVersion`] (`2.4.4`).
    #[inline]
    fn version(&self) -> Result<Option<&U16CStr>> {
        check_capability_result_from_context(
            WSLContext::get_current(),
            WSLVersionCapability::DistributionVersion,
        )?;
        // SAFETY: The WSL Plugin API guarantees that a non-null Version remains valid for the
        // lifetime of this distribution information.
        Ok(unsafe { opt_wide_str(std::ptr::addr_of!((*self.as_ptr()).Version).read_unaligned()) })
    }
}

impl<T> PartialEq<T> for WSLOfflineDistributionInformation
where
    T: CoreWSLDistributionInformation,
{
    /// Compares two distributions by their IDs for equality.
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.id() == other.id()
    }
}

impl Hash for WSLOfflineDistributionInformation {
    /// Computes a hash based on the distribution's ID.
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Display for WSLOfflineDistributionInformation {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {{{}}}", self.name().display(), self.id())
    }
}

impl Debug for WSLOfflineDistributionInformation {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct(stringify!(WSLOfflineDistributionInformation));
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
    use wslpluginapi_sys::windows_sys::core::GUID;

    #[repr(C)]
    struct BaselineOfflineDistribution {
        id: GUID,
        name: *const u16,
        package_family_name: *const u16,
    }

    #[test]
    fn reads_offline_prefix_without_flavor_or_version() {
        let name = [u16::from(b'D'), 0];
        let raw = BaselineOfflineDistribution {
            id: GUID {
                data1: 43,
                data2: 0,
                data3: 0,
                data4: [0; 8],
            },
            name: name.as_ptr(),
            package_family_name: std::ptr::null(),
        };
        // SAFETY: This is the original offline layout with live immutable string storage.
        let view =
            unsafe { WSLOfflineDistributionInformation::from_raw(std::ptr::from_ref(&raw).cast()) };
        assert_eq!(view.id(), UserDistributionID::from(raw.id));
        assert_eq!(view.name().as_slice(), &name[..1]);
        assert!(view.package_family_name().is_none());
    }
}
