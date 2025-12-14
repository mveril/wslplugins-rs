use std::{
    fmt::{LowerHex, UpperHex},
    io::Write,
    str::FromStr,
};

use crate::user_distribution_id::ParseError;
use windows_core::GUID;

use crate::{user_distribution_id::fmt::formatter::Formatter, UserDistributionID};
pub struct GuidFormatter(GUID);

impl GuidFormatter {
    const ENCODE_BUFFER: [u8; 36] = [0u8; 36];
}

impl From<UserDistributionID> for GuidFormatter {
    #[inline]
    fn from(value: UserDistributionID) -> Self {
        GUID::from(value).into()
    }
}

impl From<GUID> for GuidFormatter {
    #[inline]
    fn from(value: GUID) -> Self {
        Self(value)
    }
}

impl Formatter for GuidFormatter {}

impl UpperHex for GuidFormatter {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl LowerHex for GuidFormatter {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = Self::ENCODE_BUFFER;
        write!(&mut buffer[..], "{:?}", self.0).map_err(|_| std::fmt::Error)?;
        for b in &mut buffer {
            *b = b.to_ascii_lowercase();
        }
        // SAFETY: All bytes are valid ASCII characters.
        let s = unsafe { core::str::from_utf8_unchecked(&buffer) };
        f.write_str(s)
    }
}

impl FromStr for GuidFormatter {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GUID::try_from(s).map(|guid| Self(guid))?)
    }
}
