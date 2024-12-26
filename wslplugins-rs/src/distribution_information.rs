extern crate wslplugins_sys;
use crate::api::{
    errors::require_update_error::Result, utils::check_required_version_result_from_context,
};
use crate::core_distribution_information::CoreDistributionInformation;
use crate::WSLContext;
use std::fmt::Debug;
use std::hash::Hash;
use std::{ffi::OsString, fmt::Display, os::windows::ffi::OsStringExt};
use windows::core::GUID;
use wslplugins_sys::WSLVersion;
pub struct DistributionInformation<'a>(&'a wslplugins_sys::WSLDistributionInformation);

impl<'a> From<&'a wslplugins_sys::WSLDistributionInformation> for DistributionInformation<'a> {
    fn from(ptr: &'a wslplugins_sys::WSLDistributionInformation) -> Self {
        Self(ptr)
    }
}

impl DistributionInformation<'_> {
    /// Pid of the init process. Introduced in 2.0.5
    pub fn init_pid(&self) -> Result<u32> {
        check_required_version_result_from_context(
            WSLContext::get_current_or_panic(),
            &WSLVersion::new(2, 0, 5),
        )?;
        Ok(self.0.InitPid)
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

impl<T> PartialEq<T> for DistributionInformation<'_>
where
    T: CoreDistributionInformation,
{
    fn eq(&self, other: &T) -> bool {
        self.id() == other.id()
    }
}

impl Hash for DistributionInformation<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Display for DistributionInformation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "{:} {{{:?}}}", self.0.Name.display(), self.0.Id) }
    }
}

impl Debug for DistributionInformation<'_> {
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
