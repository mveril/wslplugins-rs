use std::{
    fmt::{self, Debug, Display},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionID(pub u32);

impl From<u32> for SessionID {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<SessionID> for u32 {
    #[inline]
    fn from(value: SessionID) -> Self {
        value.0
    }
}

impl Debug for SessionID {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for SessionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}
