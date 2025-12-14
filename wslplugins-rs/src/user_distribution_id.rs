use std::{
    fmt::{Debug, Display, LowerHex, UpperHex},
    str::FromStr,
};
pub mod fmt;
mod parse_error;
use fmt::DefaultFormatter;
pub use parse_error::ParseError;
#[cfg(feature = "uuid")]
mod uuid_impl;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UserDistributionID(pub windows_core::GUID);

impl From<windows_core::GUID> for UserDistributionID {
    #[inline]
    fn from(value: windows_core::GUID) -> Self {
        Self(value)
    }
}

impl From<wslpluginapi_sys::windows_sys::core::GUID> for UserDistributionID {
    #[inline]
    fn from(value: wslpluginapi_sys::windows_sys::core::GUID) -> Self {
        // SAFETY: Representation is the same
        let guid = unsafe {
            std::mem::transmute::<wslpluginapi_sys::windows_sys::core::GUID, windows_core::GUID>(
                value,
            )
        };
        Self(guid)
    }
}

impl From<UserDistributionID> for wslpluginapi_sys::windows_sys::core::GUID {
    #[inline]
    fn from(value: UserDistributionID) -> Self {
        // SAFETY: Representation is the same
        unsafe { std::mem::transmute(value.0) }
    }
}

impl From<UserDistributionID> for windows_core::GUID {
    #[inline]
    fn from(value: UserDistributionID) -> Self {
        value.0
    }
}

impl UpperHex for UserDistributionID {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&DefaultFormatter::from(*self), f)
    }
}

impl LowerHex for UserDistributionID {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&DefaultFormatter::from(*self), f)
    }
}

impl Display for UserDistributionID {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self, f)
    }
}

#[cfg(any(windows, feature = "uuid"))]
impl FromStr for UserDistributionID {
    type Err = ParseError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DefaultFormatter::from_str(s).map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn test_user_distribution_id_formating(int_value in any::<u128>()) {
            let guid = windows_core::GUID::from_u128(int_value);
            let user_dist_id: UserDistributionID = guid.into();
            prop_assert_eq!(format!("{:?}", guid), format!("{:X}", user_dist_id));
            prop_assert_eq!(format!("{:X}", user_dist_id), format!("{:}", user_dist_id));
            prop_assert_eq!(format!("{:X}", user_dist_id), format!("{:?}", user_dist_id));
            prop_assert_eq!(format!("{:x}", user_dist_id), format!("{user_dist_id:X}").to_ascii_lowercase());
        }
    }
}
