extern crate wslplugins_sys;
use crate::core_distribution_information::CoreDistributionInformation;
use std::{ffi::OsString, os::windows::ffi::OsStringExt};
use windows::core::GUID;

pub struct OfflineDistributionInformation<'a>(
    &'a wslplugins_sys::WslOfflineDistributionInformation,
);

impl<'a> OfflineDistributionInformation<'a> {
    pub fn from(ptr: &'a wslplugins_sys::WslOfflineDistributionInformation) -> Self {
        Self(ptr)
    }
}

impl<'a> CoreDistributionInformation for OfflineDistributionInformation<'a> {
    fn id(&self) -> &GUID {
        &self.0.Id
    }

    fn name(&self) -> OsString {
        unsafe { OsString::from_wide(self.0.Name.as_wide()) }
    }

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
