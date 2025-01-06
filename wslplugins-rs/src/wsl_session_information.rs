//! # WSL Session Information
//!
//! This module provides a safe abstraction over the `WSLSessionInformation` structure
//! from the WSL Plugin API, allowing access to session details in an idiomatic Rust interface.

extern crate wslplugins_sys;
use core::hash;
use std::fmt;
use windows::Win32::Foundation::*;
use windows::Win32::Security::PSID;

/// Represents session information for a WSL instance.
///
/// This struct wraps the `WSLSessionInformation` provided by the WSL Plugin API and
/// provides safe, idiomatic access to its fields.
///
/// # Lifetime Parameters
/// - `'a`: The lifetime of the referenced `WSLSessionInformation` instance.
pub struct WSLSessionInformation<'a>(&'a wslplugins_sys::WSLSessionInformation);

impl WSLSessionInformation<'_> {
    /// Retrieves the session ID.
    ///
    /// # Returns
    /// The unique session ID as a [u32].
    pub fn id(&self) -> u32 {
        self.0.SessionId
    }

    /// Retrieves the user token for the session.
    ///
    /// # Returns
    /// A [HANDLE] representing the user token.
    pub fn user_token(&self) -> HANDLE {
        self.0.UserToken
    }

    /// Retrieves the user SID (security identifier) for the session.
    ///
    /// # Returns
    /// A [PSID] representing the user SID.
    pub fn user_sid(&self) -> PSID {
        self.0.UserSid
    }
}

impl<'a> From<&'a wslplugins_sys::WSLSessionInformation> for WSLSessionInformation<'a> {
    /// Creates a `WSLSessionInformation` instance from a reference to `WSLSessionInformation` from the API.
    ///
    /// # Arguments
    /// - `ptr`: A reference to a `WSLSessionInformation` instance.
    ///
    /// # Returns
    /// A safe wrapper around the provided pointer.
    fn from(ptr: &'a wslplugins_sys::WSLSessionInformation) -> Self {
        Self(ptr)
    }
}

impl hash::Hash for WSLSessionInformation<'_> {
    /// Computes a hash based on the session ID.
    ///
    /// # Arguments
    /// - `state`: The hasher state to update with the session ID.
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.SessionId.hash(state);
    }
}

impl PartialEq for WSLSessionInformation<'_> {
    /// Compares two `WSLSessionInformation` instances for equality based on their session IDs.
    ///
    /// # Arguments
    /// - `other`: The other `WSLSessionInformation` instance to compare.
    ///
    /// # Returns
    /// `true` if the session IDs are equal, `false` otherwise.
    fn eq(&self, other: &Self) -> bool {
        self.0.SessionId == other.0.SessionId
    }
}

// Manually implements Debug for `WSLSessionInformation`.
impl fmt::Debug for WSLSessionInformation<'_> {
    /// Formats the session information for debugging.
    ///
    /// The output includes the session ID, user token, and user SID.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WSLSessionInformation")
            .field("sessionId", &self.0.SessionId)
            .field("userToken", &self.0.UserToken)
            .field("userSid", &self.0.UserSid)
            .finish()
    }
}
