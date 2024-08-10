extern crate wslplugins_sys;
use std::{ffi::OsString, os::windows::ffi::OsStringExt};
use windows::core::GUID;
use crate::core_distribution_information::CoreDistributionInformation;


pub struct OfflineDistributionInformation(*const wslplugins_sys::WslOfflineDistributionInformation);

impl OfflineDistributionInformation {
    pub fn from_raw(ptr: *const wslplugins_sys::WslOfflineDistributionInformation) -> Self {
      Self(ptr)
    }
}

impl CoreDistributionInformation for OfflineDistributionInformation {
    fn id(&self) -> &GUID {
      unsafe {
        &(*self.0).Id
      }
    }

    fn name(&self) -> OsString {
      unsafe {
        OsString::from_wide((*self.0).Name.as_wide())
      }
    }

    fn package_family_name(&self) -> Option<OsString> {
        unsafe {  
          let ptr = (*self.0).PackageFamilyName;
          if ptr.is_null() || ptr.is_empty() {
              None
          }
          else {
              Some(OsString::from_wide(ptr.as_wide()))
          }
        }
    }
}