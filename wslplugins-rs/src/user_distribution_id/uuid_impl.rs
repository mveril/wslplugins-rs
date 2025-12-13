use std::str::FromStr;

use super::UserDistributionID;
use uuid::Uuid;

impl From<UserDistributionID> for Uuid {
    #[inline]
    fn from(value: UserDistributionID) -> Self {
        Self::from_u128(value.0.to_u128())
    }
}

impl From<Uuid> for UserDistributionID {
    #[inline]
    fn from(value: Uuid) -> Self {
        Self(windows_core::GUID::from_u128(value.as_u128()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use uuid::{fmt::Hyphenated, Uuid};
    proptest! {
        #[test]
        fn test_uuid_user_distribution_id_conversion(int_value in any::<u128>()) {
            let guid = windows_core::GUID::from_u128(int_value);
            let user_dist_id: UserDistributionID = guid.into();
            let converted_uuid: Uuid = user_dist_id.into();
            prop_assert_eq!(format!("{:}", user_dist_id), format!("{:X}", Hyphenated::from(converted_uuid)));
            prop_assert_eq!(format!("{:X}", user_dist_id), format!("{:X}", Hyphenated::from(converted_uuid)));
            prop_assert_eq!(format!("{:x}", user_dist_id), format!("{:x}", Hyphenated::from(converted_uuid)));
        }
    }
}

