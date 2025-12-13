use crate::user_distribution_id::ParseError;
use crate::user_distribution_id::{fmt::formatter::Formatter, UserDistributionID};
use std::{
    fmt::{LowerHex, Result, UpperHex},
    str::FromStr,
};
use uuid::{fmt::Hyphenated, Uuid};
pub struct UuidFormatter(Uuid);

impl Formatter for UuidFormatter {}

impl LowerHex for UuidFormatter {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        LowerHex::fmt(&self.0.as_hyphenated(), f)
    }
}

impl UpperHex for UuidFormatter {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        LowerHex::fmt(&self.0.as_hyphenated(), f)
    }
}

impl From<&UserDistributionID> for UuidFormatter {
    #[inline]
    fn from(value: &UserDistributionID) -> Self {
        let uuid: uuid::Uuid = (*value).into();
        Self(uuid)
    }
}

impl From<Uuid> for UuidFormatter {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl FromStr for UuidFormatter {
    type Err = ParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Uuid::parse_str(s).map(|uuid| Self(uuid))?)
    }
}
