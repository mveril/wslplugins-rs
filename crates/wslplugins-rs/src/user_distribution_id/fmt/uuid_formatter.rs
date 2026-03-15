use crate::user_distribution_id::ParseError;
use crate::user_distribution_id::{fmt::formatter::Formatter, UserDistributionID};
use std::{
    fmt::{LowerHex, Result, UpperHex},
    str::FromStr,
};
use uuid::Uuid;
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
        UpperHex::fmt(&self.0.as_hyphenated(), f)
    }
}

impl From<UserDistributionID> for UuidFormatter {
    #[inline]
    fn from(value: UserDistributionID) -> Self {
        Self(Uuid::from(value))
    }
}

impl From<UuidFormatter> for UserDistributionID {
    #[inline]
    fn from(value: UuidFormatter) -> Self {
        value.0.into()
    }
}

impl FromStr for UuidFormatter {
    type Err = ParseError;
    #[inline]
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Uuid::parse_str(s).map(Self)?)
    }
}
