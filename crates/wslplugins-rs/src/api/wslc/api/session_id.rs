/// Identifies a WSL container session.
///
/// WSLC session identifiers are a different identifier space from regular WSL
/// session identifiers, so this type deliberately does not convert to or from
/// [`crate::SessionID`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct WSLCSessionID(u32);

impl WSLCSessionID {
    /// Creates an identifier from its native value.
    #[must_use]
    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the native identifier value.
    #[must_use]
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for WSLCSessionID {
    #[inline]
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl From<WSLCSessionID> for u32 {
    #[inline]
    fn from(value: WSLCSessionID) -> Self {
        value.get()
    }
}
