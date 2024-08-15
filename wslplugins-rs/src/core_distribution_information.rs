use std::{
    ffi::OsString,
    fmt::{Debug, Display},
    hash,
};
use windows::core::GUID;

pub trait CoreDistributionInformation {
    /// Distribution ID, guaranteed to be the same accross reboots
    fn id(&self) -> &GUID;
    fn name(&self) -> OsString;
    /// Package family name, if the distribution is packaged
    fn package_family_name(&self) -> Option<OsString>;
}

impl hash::Hash for &dyn CoreDistributionInformation {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl PartialEq for &dyn CoreDistributionInformation {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Display for &dyn CoreDistributionInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name().fmt(f)
    }
}

impl Debug for &dyn CoreDistributionInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.name().to_string_lossy(), self.id(),)?;

        if let Some(package) = self.package_family_name() {
            write!(f, " [{:?}]", package)?;
        }

        Ok(())
    }
}
