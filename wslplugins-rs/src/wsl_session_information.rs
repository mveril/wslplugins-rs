extern crate wslplugins_sys;
use core::hash;
use std::fmt;
use windows::Win32::Foundation::*;
use windows::Win32::Security::PSID;

pub struct WSLSessionInformation(*const wslplugins_sys::WSLSessionInformation);

impl hash::Hash for WSLSessionInformation {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        unsafe { (*self.0).SessionId.hash(state); }
    }
}

impl PartialEq for WSLSessionInformation {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (*self.0).SessionId == (*other.0).SessionId }
    }
}

impl WSLSessionInformation {
  
    // Getter for id
    pub fn id(&self) -> u32 {
        unsafe { (*self.0).SessionId }
    }

    // Getter for user_token
    pub fn user_token(&self) -> HANDLE {
        unsafe { (*self.0).UserToken }
    }

    // Getter for user_sid
    pub fn user_sid(&self) -> &PSID {
        unsafe { &(*self.0).UserSid }
    }

    pub fn from_raw(native: *const wslplugins_sys::WSLSessionInformation) -> Self {
        Self(native)
    }
}

// Implement Debug manually
impl fmt::Debug for WSLSessionInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            f.debug_struct("WSLSessionInformation")
                .field("SessionId", &(*self.0).SessionId)
                .field("UserToken", &(*self.0).UserToken)
                .field("UserSid", &(*self.0).UserSid)
                .finish()
        }
    }
}
