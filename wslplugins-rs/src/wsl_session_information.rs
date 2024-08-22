extern crate wslplugins_sys;
use core::hash;
use std::fmt;
use windows::Win32::Foundation::*;
use windows::Win32::Security::PSID;

pub struct WSLSessionInformation<'a>(&'a wslplugins_sys::WSLSessionInformation);

impl hash::Hash for WSLSessionInformation<'_> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.SessionId.hash(state);
    }
}

impl PartialEq for WSLSessionInformation<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.SessionId == other.0.SessionId
    }
}

impl WSLSessionInformation<'_> {
    // Getter for id
    pub fn id(&self) -> u32 {
        self.0.SessionId
    }

    // Getter for user_token
    pub fn user_token(&self) -> HANDLE {
        self.0.UserToken
    }

    // Getter for user_sid
    pub fn user_sid(&self) -> PSID {
        self.0.UserSid
    }
}

impl<'a> From<&'a wslplugins_sys::WSLSessionInformation> for WSLSessionInformation<'a> {
    fn from(ptr: &'a wslplugins_sys::WSLSessionInformation) -> Self {
        Self(ptr)
    }
}

// Implement Debug manually
impl fmt::Debug for WSLSessionInformation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WSLSessionInformation")
            .field("SessionId", &(*self.0).SessionId)
            .field("UserToken", &(*self.0).UserToken)
            .field("UserSid", &(*self.0).UserSid)
            .finish()
    }
}
