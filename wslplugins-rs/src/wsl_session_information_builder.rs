use std::process;
use windows::core::Error;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    CopySid, GetLengthSid, GetTokenInformation, TokenUser, PSID, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::System::Memory::{LocalAlloc, LMEM_FIXED, LMEM_ZEROINIT};
use windows::Win32::System::SystemServices::LocalFree;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use wslpluginapi_sys::WSLSessionId;

/// Builder pattern for constructing `WSLSessionInformation`.
pub struct WSLSessionInformationBuilder {
    session_id: WSLSessionId,
    user_token: HANDLE,
    user_sid: PSID,
}

impl WSLSessionInformationBuilder {
    /// Creates a new builder instance.
    pub fn new() -> Self {
        Self {
            session_id: Default::default(),
            user_token: Default::default(),
            user_sid: Default::default(),
        }
    }

    /// Sets the session ID.
    pub fn session_id(mut self, id: WSLSessionId) -> Self {
        self.session_id = Some(id);
        self
    }

    /// Retrieves the current user token and duplicates the user SID.
    ///
    /// While `std::process::id()` provides the process ID, accessing the process handle
    /// and obtaining user authentication details requires the Windows API.
    pub fn with_current_user(mut self) -> Result<Self, Error> {
        unsafe {
            // Get the current process token.
            let mut token = HANDLE::default();
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

            // Determine the required buffer size for TOKEN_USER.
            let mut len = 0u32;
            GetTokenInformation(token, TokenUser, None, 0, &mut len)?;

            // Allocate a buffer and retrieve the token user information.
            let mut buf = vec![0u8; len as usize];
            GetTokenInformation(
                token,
                TokenUser,
                Some(buf.as_mut_ptr() as *mut _),
                len,
                &mut len,
            )?;

            let token_user = &*(buf.as_ptr() as *const TOKEN_USER);
            let sid = token_user.User.Sid;
            let sid_len = GetLengthSid(sid) as usize;

            // Allocate memory for the copied SID.
            let allocated_sid = LocalAlloc(LMEM_FIXED | LMEM_ZEROINIT, sid_len)?;

            // Copy the SID into the allocated memory.
            CopySid(sid_len, allocated_sid, sid)?;
            

            self.user_token = token;
            self.user_sid = sid.;
        }
        Ok(self)
    }

    /// Builds the `WSLSessionInformation` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if one or more fields are missing.
    pub fn build(self) -> Result<WSLSessionInformation, &'static str> {
        match (self.session_id, self.user_token, self.user_sid) {
            (Some(id), Some(token), Some(sid)) => Ok(WSLSessionInformation {
                SessionId: id,
                UserToken: token,
                UserSid: sid,
            }),
            _ => Err("One or more fields are missing"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Log the current process ID using the standard library.
    println!("Current process ID (std): {}", process::id());

    // Example usage: assuming a session ID of 1.
    let session_info = WSLSessionInformationBuilder::new()
        .session_id(1)
        .with_current_user()?
        .build()?;
    println!("Session built with ID: {}", session_info.SessionId);

    // Free allocated resources.
    unsafe {
        CloseHandle(session_info.UserToken);
        LocalFree(session_info.UserSid);
    }
    Ok(())
}
