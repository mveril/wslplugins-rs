extern crate wslplugins_sys;
use crate::core_distribution_information::CoreDistributionInformation;
use std::{ffi::OsString, os::windows::ffi::OsStringExt};
use windows::core::GUID;

pub struct DistributionInformation<'a>(&'a wslplugins_sys::WSLDistributionInformation);

impl<'a> From<&'a wslplugins_sys::WSLDistributionInformation> for DistributionInformation<'a> {
    fn from(ptr: &'a wslplugins_sys::WSLDistributionInformation) -> Self {
        Self(ptr)
    }
}

impl DistributionInformation<'_> {
    /// Pid of the init process. Introduced in 2.0.5
    pub fn init_pid(&self) -> u32 {
        self.0.InitPid
    }

    pub fn pid_namespace(&self) -> u64 {
        self.0.PidNamespace
    }
}

impl CoreDistributionInformation for DistributionInformation<'_> {
    fn id(&self) -> &GUID {
        &self.0.Id
    }

    fn name(&self) -> OsString {
        unsafe { OsString::from_wide(self.0.Name.as_wide()) }
    }

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
