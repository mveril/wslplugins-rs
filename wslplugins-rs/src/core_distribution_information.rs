use std::ffi::OsString;
use windows::core::GUID;

pub trait CoreDistributionInformation {
    /// Distribution ID, guaranteed to be the same accross reboots
    fn id(&self) -> &GUID;
    fn name(&self) -> OsString;
    /// Package family name, if the distribution is packaged
    fn package_family_name(&self) -> Option<OsString>;
}
