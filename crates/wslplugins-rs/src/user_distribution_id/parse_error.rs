use thiserror::Error;
use windows_core::HRESULT;

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum ParseError {
    #[error("Windows error: HRESULT={0}")]
    Windows(HRESULT),

    #[cfg(feature = "uuid")]
    #[error("UUID parsing error: {0}")]
    UUID(#[from] uuid::Error),
}

impl From<windows_core::Error> for ParseError {
    #[inline]
    fn from(value: windows_core::Error) -> Self {
        Self::Windows(value.code())
    }
}
#[cfg(feature = "uuid")]
#[expect(
    clippy::unreadable_literal,
    reason = "It's the natural Microsoft error format"
)]
pub const E_INVALIDARG: windows_core::HRESULT = HRESULT(0x80070057_u32.cast_signed());
impl From<ParseError> for windows_core::HRESULT {
    #[inline]
    fn from(value: ParseError) -> Self {
        match value {
            ParseError::Windows(hresult) => hresult,
            #[cfg(feature = "uuid")]
            ParseError::UUID(_) => E_INVALIDARG,
        }
    }
}

impl From<ParseError> for windows_core::Error {
    #[inline]
    fn from(value: ParseError) -> Self {
        Self::from_hresult(value.into())
    }
}
