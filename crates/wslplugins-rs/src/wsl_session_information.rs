//! # WSL Session Information
//!
//! This module provides a safe abstraction over the `WSLSessionInformation` structure
//! from the WSL Plugin API, allowing access to session details in an idiomatic Rust interface.

use crate::{HasSessionId, SessionID};
use core::hash;
use std::{fmt, os::windows::raw::HANDLE};
use wslpluginapi_sys::windows_sys::Win32::Security::PSID;

/// Represents session information for a WSL instance.
///
/// This struct wraps the `WSLSessionInformation` provided by the WSL Plugin API and
/// provides safe, idiomatic access to its fields.
#[repr(transparent)]
pub struct WSLSessionInformation(wslpluginapi_sys::WSLSessionInformation);

impl WSLSessionInformation {
    /// Retrieves the session ID.
    ///
    /// # Returns
    /// The unique session ID as a [`SessionID`].
    #[must_use]
    #[inline]
    pub const fn id(&self) -> SessionID {
        SessionID(self.0.SessionId)
    }

    /// Retrieves the user token for the session.
    ///
    /// # Returns
    /// A [HANDLE] representing the user token.
    /// # Safety
    /// This function returns a raw handle to the user token.
    /// The handle should be used only during the life of the session and must not be closed
    #[must_use]
    #[inline]
    pub const unsafe fn user_token(&self) -> HANDLE {
        self.0.UserToken
    }

    /// Retrieves the user SID (security identifier) for the session.
    ///
    /// # Returns
    /// A [PSID] representing the user SID.
    /// # Safety
    /// This function returns a raw pointer to the user SID.
    /// This pointer should be used only during the life of the session and must not be freed or modified.
    #[must_use]
    #[inline]
    pub const unsafe fn user_sid(&self) -> PSID {
        self.0.UserSid
    }
}

impl HasSessionId for WSLSessionInformation {
    #[inline]
    fn session_id(&self) -> SessionID {
        self.id()
    }
}

impl From<wslpluginapi_sys::WSLSessionInformation> for WSLSessionInformation {
    #[inline]
    fn from(value: wslpluginapi_sys::WSLSessionInformation) -> Self {
        Self(value)
    }
}

impl From<WSLSessionInformation> for wslpluginapi_sys::WSLSessionInformation {
    #[inline]
    fn from(value: WSLSessionInformation) -> Self {
        value.0
    }
}

impl AsRef<WSLSessionInformation> for wslpluginapi_sys::WSLSessionInformation {
    #[inline]
    fn as_ref(&self) -> &WSLSessionInformation {
        // SAFETY: conveting this kind of ref is safe as it is transparent
        unsafe { &*std::ptr::from_ref::<Self>(self).cast::<WSLSessionInformation>() }
    }
}

impl AsRef<wslpluginapi_sys::WSLSessionInformation> for WSLSessionInformation {
    #[inline]
    fn as_ref(&self) -> &wslpluginapi_sys::WSLSessionInformation {
        &self.0
    }
}

impl hash::Hash for WSLSessionInformation {
    /// Computes a hash based on the session ID.
    ///
    /// # Arguments
    /// - `state`: The hasher state to update with the session ID.
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.SessionId.hash(state);
    }
}

impl PartialEq for WSLSessionInformation {
    /// Compares two `WSLSessionInformation` instances for equality based on their session IDs.
    ///
    /// # Arguments
    /// - `other`: The other `WSLSessionInformation` instance to compare.
    ///
    /// # Returns
    /// `true` if the session IDs are equal, `false` otherwise.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.SessionId == other.0.SessionId
    }
}

// Manually implements Debug for `WSLSessionInformation`.
impl fmt::Debug for WSLSessionInformation {
    /// Formats the session information for debugging.
    ///
    /// The output includes the session ID, user token, and user SID.
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WSLSessionInformation")
            .field("sessionId", &self.0.SessionId)
            .field("userToken", &self.0.UserToken)
            .field("userSid", &self.0.UserSid)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::test_transparence;

    use super::WSLSessionInformation;

    #[test]
    fn test_layouts() {
        test_transparence::<wslpluginapi_sys::WSLSessionInformation, WSLSessionInformation>();
    }
}
