use thiserror::Error;
use windows_core::HRESULT;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Windows error: HRESULT={0}")]
    Windows(HRESULT),

    #[cfg(feature = "uuid")]
    #[error("UUID parsing error: {0}")]
    UUID(#[from] uuid::Error),
}

impl From<windows_core::Error> for ParseError {
    fn from(value: windows_core::Error) -> Self {
        Self::Windows(value.code())
    }
}
pub const E_INVALIDARG: windows_core::HRESULT = HRESULT(0x80070057_u32 as _);
impl From<ParseError> for windows_core::HRESULT {
    fn from(value: ParseError) -> Self {
        match value {
            ParseError::Windows(hresult) => hresult,
            ParseError::UUID(error) => E_INVALIDARG,
        }
    }
}

impl From<ParseError> for windows_core::Error {
    fn from(value: ParseError) -> Self {
        Self::from_hresult(value.into())
    }
}
