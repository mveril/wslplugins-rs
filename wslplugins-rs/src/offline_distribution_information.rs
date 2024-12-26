extern crate wslplugins_sys;
use crate::core_distribution_information::CoreDistributionInformation;
use std::{
    ffi::OsString,
    fmt::{Debug, Display},
    hash::Hash,
    os::windows::ffi::OsStringExt,
};
use windows::core::GUID;

pub struct OfflineDistributionInformation<'a>(
    &'a wslplugins_sys::WslOfflineDistributionInformation,
);

impl<'a> OfflineDistributionInformation<'a> {
    pub fn from(ptr: &'a wslplugins_sys::WslOfflineDistributionInformation) -> Self {
        Self(ptr)
    }
}

impl CoreDistributionInformation for OfflineDistributionInformation<'_> {
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

impl<T> PartialEq<T> for OfflineDistributionInformation<'_>
where
    T: CoreDistributionInformation,
{
    fn eq(&self, other: &T) -> bool {
        self.id() == other.id()
    }
}

impl Hash for OfflineDistributionInformation<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Display for OfflineDistributionInformation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "{:} {{{:?}}}", self.0.Name.display(), self.0.Id) }
    }
}

impl Debug for OfflineDistributionInformation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DistributionInformation")
            .field("name", &self.name())
            .field("id", &self.id())
            .field("package_family_name", &self.package_family_name())
            .finish()
    }
}
