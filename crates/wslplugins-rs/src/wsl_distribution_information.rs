//! # WSL Distribution Information
//!
//! This module provides a safe abstraction for accessing information about a WSL distribution.
//! It wraps the [`wslpluginapi_sys::WSLDistributionInformation`] structure from the WSL Plugin API and implements
//! the [`CoreWSLDistributionInformation`] trait for consistent access to distribution details.
//!
//! ## Overview
//! The `WSLDistributionInformation` struct provides methods to retrieve:
//! - Distribution ID
//! - Distribution name
//! - Package family name (if applicable)
//! - Process ID (PID) of the init process (requires
//!   [`WSLVersionCapability::DistributionInitPid`] (`2.0.5`) capability)
//! - PID namespace

#[cfg(doc)]
use crate::api::errors::require_update_error::Error;
use crate::api::{
    errors::require_update_error::Result, utils::check_capability_result_from_context,
};
use crate::core_wsl_distribution_information::CoreWSLDistributionInformation;
use crate::utils::{opt_wide_str, wide_str};
use crate::{UserDistributionID, WSLContext, WSLVersionCapability};
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use widestring::U16CStr;

/// Represents detailed information about a WSL distribution.
///
/// This struct wraps the `WSLDistributionInformation` from the WSL Plugin API and provides
/// safe, idiomatic Rust access to its fields.
// Only the fields supported by the host version may be read through this view.
#[repr(C)]
pub struct WSLDistributionInformation {
    _private: (),
    _marker: std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
}

impl WSLDistributionInformation {
    /// Borrows a versioned WSL structure without referencing its full current layout.
    ///
    /// # Safety
    /// `raw` must be non-null, correctly aligned, and point to a WSL-provided structure, including all
    /// fields supported by the current host version. The structure and its pointees
    /// must remain valid and immutable for `'a`. Distribution views must not outlive
    /// their callback, and versioned access requires the matching initialized context.
    #[inline]
    #[must_use]
    pub const unsafe fn from_raw<'a>(
        raw: *const wslpluginapi_sys::WSLDistributionInformation,
    ) -> &'a Self {
        // SAFETY: The caller guarantees a valid allocation for this opaque borrowed view.
        unsafe { &*raw.cast::<Self>() }
    }

    /// Returns the pointer used for field-by-field ABI access.
    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const wslpluginapi_sys::WSLDistributionInformation {
        std::ptr::from_ref(self).cast()
    }
}

impl WSLDistributionInformation {
    /// Retrieves the PID of the init process.
    ///
    /// This requires [`WSLVersionCapability::DistributionInitPid`] (`2.0.5`). If the current API
    /// version does not support the capability, an error is returned.
    ///
    /// # Returns
    /// - `Ok(pid)`: The PID of the init process.
    /// # Errors
    /// [Error]: If the runtime API capability is insufficient.
    #[inline]
    pub fn init_pid(&self) -> Result<u32> {
        check_capability_result_from_context(
            WSLContext::get_current(),
            WSLVersionCapability::DistributionInitPid,
        )?;
        // SAFETY: The capability check above guarantees that this field exists.
        Ok(unsafe { std::ptr::addr_of!((*self.as_ptr()).InitPid).read_unaligned() })
    }

    /// Retrieves the PID namespace for the distribution.
    ///
    /// # Returns
    /// The PID namespace as a `u64`.
    ///
    #[inline]
    #[must_use]
    pub const fn pid_namespace(&self) -> u64 {
        // SAFETY: This field is part of the baseline ABI guaranteed by from_raw.
        unsafe { std::ptr::addr_of!((*self.as_ptr()).PidNamespace).read_unaligned() }
    }
}

impl CoreWSLDistributionInformation for WSLDistributionInformation {
    #[inline]
    fn id(&self) -> UserDistributionID {
        // SAFETY: This field is part of the baseline ABI guaranteed by from_raw.
        unsafe { std::ptr::addr_of!((*self.as_ptr()).Id).read_unaligned() }.into()
    }

    #[inline]
    fn name(&self) -> &U16CStr {
        // SAFETY: The WSL Plugin API guarantees that Name points to a valid null-terminated
        // UTF-16 string for the lifetime of this distribution information.
        unsafe { wide_str(std::ptr::addr_of!((*self.as_ptr()).Name).read_unaligned()) }
    }

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
        // SAFETY: The capability check above guarantees that this field exists.
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
        // SAFETY: The capability check above guarantees that this field exists.
        Ok(unsafe { opt_wide_str(std::ptr::addr_of!((*self.as_ptr()).Version).read_unaligned()) })
    }
}

impl<T> PartialEq<T> for WSLDistributionInformation
where
    T: CoreWSLDistributionInformation,
{
    /// Compares two distributions for equality based on their IDs.
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.id() == other.id()
    }
}

impl Hash for WSLDistributionInformation {
    /// Computes a hash based on the distribution's ID.
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Display for WSLDistributionInformation {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: Name is known to be valid
        write!(f, "{} {{{}}}", self.name().display(), self.id())
    }
}

impl Debug for WSLDistributionInformation {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct(stringify!(WSLDistributionInformation));
        dbg.field("name", &self.name())
            .field("id", &self.id())
            .field("package_family_name", &self.package_family_name())
            .field("pid_namespace", &self.pid_namespace());
        let mut exhaustive = true;

        if let Ok(pid) = self.init_pid() {
            dbg.field("init_pid", &pid);
        } else {
            exhaustive = false;
        }
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

    // WSL 2.0.0 did not allocate InitPid, Flavor, or Version.
    #[repr(C)]
    struct BaselineDistribution {
        id: GUID,
        name: *const u16,
        pid_namespace: u64,
        package_family_name: *const u16,
    }

    #[test]
    fn reads_distribution_prefix_without_newer_fields() {
        let name = [u16::from(b'D'), 0];
        let raw = BaselineDistribution {
            id: GUID {
                data1: 42,
                data2: 0,
                data3: 0,
                data4: [0; 8],
            },
            name: name.as_ptr(),
            pid_namespace: 123,
            package_family_name: std::ptr::null(),
        };
        // SAFETY: This is the baseline host layout, with live immutable string storage.
        let view = unsafe { WSLDistributionInformation::from_raw(std::ptr::from_ref(&raw).cast()) };
        assert_eq!(view.id(), UserDistributionID::from(raw.id));
        assert_eq!(view.name().as_slice(), &name[..1]);
        assert_eq!(view.pid_namespace(), 123);
        assert!(view.package_family_name().is_none());
        assert_eq!(
            view.as_ptr().cast::<BaselineDistribution>(),
            std::ptr::from_ref(&raw)
        );
    }
}
