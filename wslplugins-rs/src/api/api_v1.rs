#[cfg(doc)]
use super::Error;
use super::{Result, WSLCommand};
use crate::api::errors::require_update_error::Result as UpReqResult;
use crate::api::wsl_command::IntoCowUtf8UnixPath;
use crate::{SessionID, UserDistributionID, WSLVersion};
use std::ffi::OsStr;
use std::fmt::{self, Debug};
use std::mem::MaybeUninit;
use std::net::TcpStream;
use std::os::windows::io::FromRawSocket as _;
use std::os::windows::raw::SOCKET;
use std::path::Path;
#[cfg(feature = "tracing")]
use tracing::instrument;
use typed_path::Utf8UnixPath;
use widestring::U16CString;
use windows_core::{Result as WinResult, HRESULT};
use wslpluginapi_sys;
use wslpluginapi_sys::windows_sys::Win32::Networking::WinSock::SOCKET as WinSocket;

#[cfg(doc)]
use crate::DistributionID;

use wslpluginapi_sys::WSLPluginAPIV1;

use super::utils::check_required_version_result;

/// Represents a structured interface for interacting with the `WSLPluginAPIV1` API.
///
/// This struct encapsulates the methods provided by the `WSLPluginAPIV1` API, allowing
/// idiomatic interaction with the Windows Subsystem for Linux (WSL).
#[repr(transparent)]
pub struct ApiV1(WSLPluginAPIV1);

impl From<ApiV1> for WSLPluginAPIV1 {
    #[inline]
    fn from(value: ApiV1) -> Self {
        value.0
    }
}

impl From<WSLPluginAPIV1> for ApiV1 {
    #[inline]
    fn from(value: WSLPluginAPIV1) -> Self {
        Self(value)
    }
}

impl AsRef<WSLPluginAPIV1> for ApiV1 {
    #[inline]
    fn as_ref(&self) -> &WSLPluginAPIV1 {
        &self.0
    }
}

impl AsRef<ApiV1> for WSLPluginAPIV1 {
    #[inline]
    fn as_ref(&self) -> &ApiV1 {
        // SAFETY: The layout of ApiV1 is transparent over WSLPluginAPIV1, so this cast is safe.
        unsafe { &*std::ptr::from_ref::<Self>(self).cast::<ApiV1>() }
    }
}

impl ApiV1 {
    /// Retpurns the current version of the WSL API being used.
    ///
    /// This is useful for checking compatibility with specific API features.
    ///
    /// # Example
    /// ```ignore
    /// let api_v1: ApiV1 = ...;
    /// let version = api_v1.version();
    /// println!(
    ///     "WSL API version: {}.{}.{}",
    ///     version.Major, version.Minor, version.Revision
    /// );
    #[must_use]
    #[inline]
    pub fn version(&self) -> &WSLVersion {
        self.0.Version.as_ref()
    }

    /// Create plan9 mount between Windows & Linux
    /// Allows sharing a folder between the Windows host and the Linux environment.
    ///
    /// # Arguments
    /// - `session`: The current WSL session.
    /// - `windows_path`: The Windows path of the folder to be mounted.
    /// - `linux_path`: The Linux path where the folder will be mounted.
    /// - `read_only`: Whether the mount should be read-only.
    /// - `name`: A custom name for the mount.
    /// # Errors
    /// This function returns a windows error when the mount fails.
    /// # Example
    /// ``` rust,ignore
    /// api.mount_folder(&session, "C:\\path", "/mnt/path", false, "MyMount")?;
    /// ```
    #[doc(alias = "MountFolder")]
    #[cfg_attr(feature = "tracing", instrument(level = "trace"))]
    #[inline]
    pub fn mount_folder<
        WP: AsRef<Path> + std::fmt::Debug,
        UP: AsRef<Utf8UnixPath> + std::fmt::Debug,
    >(
        &self,
        session_id: SessionID,
        windows_path: WP,
        linux_path: UP,
        read_only: bool,
        name: &OsStr,
    ) -> WinResult<()> {
        let encoded_windows_path =
            U16CString::from_os_str_truncate(windows_path.as_ref().as_os_str());
        let encoded_linux_path = U16CString::from_str_truncate(linux_path.as_ref().as_str());
        let encoded_name = U16CString::from_os_str_truncate(name);
        // SAFETY:
        // - `self.0.MountFolder` comes from the validated `WSLPluginAPIV1` struct provided by WSL.
        //   The API guarantees that this function pointer is non-null for supported versions.
        // - All `U16CString` instances (`encoded_windows_path`, `encoded_linux_path`, `encoded_name`)
        //   ensure null-termination and valid UTF-16 encoding, so the raw pointers passed to the FFI
        //   are valid for the duration of the call.
        // - `session.id()` returns a valid `WSLSessionId` provided by WSL; it remains valid while the
        //   session is active.
        // - No aliasing or mutation of memory occurs while the function pointer is called.
        //
        // The only `unsafe` operation is the FFI call, which is trusted because it is executed under
        // WSL's documented plugin API contract.
        let result = unsafe {
            self.0.MountFolder.unwrap_unchecked()(
                u32::from(session_id),
                encoded_windows_path.as_ptr(),
                encoded_linux_path.as_ptr(),
                i32::from(read_only),
                encoded_name.as_ptr(),
            )
        };
        HRESULT(result).ok()
    }

    pub(crate) unsafe fn execute_binary_internal(
        &self,
        session_id: SessionID,
        path: &[u8],
        args: &[*const u8],
    ) -> WinResult<TcpStream> {
        let mut socket = MaybeUninit::<WinSocket>::uninit();
        // SAFETY: Calling ExecuteBinary is safe with agument correctly prepared.
        let stream = unsafe {
            HRESULT(self.0.ExecuteBinary.unwrap_unchecked()(
                u32::from(session_id),
                path.as_ptr(),
                args.as_ptr().cast_mut(),
                socket.as_mut_ptr(),
            ))
            .ok()?;

            let socket = socket.assume_init();
            TcpStream::from_raw_socket(socket as SOCKET)
        };
        Ok(stream)
    }

    /// Set the error message to display to the user if the VM or distribution creation fails.
    #[cfg_attr(feature = "tracing", instrument(level = "trace"))]
    pub(crate) fn plugin_error(&self, error: &OsStr) -> WinResult<()> {
        let error_utf16 = U16CString::from_os_str_truncate(error);
        HRESULT(
            // SAFETY: We know the pointer is always valid if the API ref is valid
            unsafe { self.0.PluginError.unwrap_unchecked()(error_utf16.as_ptr()) },
        )
        .ok()
    }

    pub(crate) unsafe fn execute_binary_in_distribution_internal(
        &self,
        session_id: SessionID,
        distribution_id: UserDistributionID,
        c_path: &[u8],
        args: &[*const u8],
    ) -> Result<TcpStream> {
        self.check_required_version(&WSLVersion::new(2, 1, 2))?;
        let mut socket = MaybeUninit::<WinSocket>::uninit();
        let guid: wslpluginapi_sys::windows_sys::core::GUID = distribution_id.into();
        // SAFETY: Calling ExecuteBinaryInDistribution is safe with agument correctly prepared.
        let stream = unsafe {
            HRESULT(self.0.ExecuteBinaryInDistribution.unwrap_unchecked()(
                u32::from(session_id),
                (&raw const guid),
                c_path.as_ptr(),
                args.as_ptr().cast_mut(),
                socket.as_mut_ptr(),
            ))
            .ok()?;

            let socket = socket.assume_init();
            TcpStream::from_raw_socket(socket as SOCKET)
        };
        Ok(stream)
    }

    /// Creates a new [`WSLCommand`] associated with this API instance.
    ///
    /// This is the preferred way to construct a command to be executed inside WSL.
    /// The returned [`WSLCommand`] is bound to:
    /// - this API handle,
    /// - the provided session,
    /// - the specified Linux program path.
    ///
    /// # Parameters
    /// - `session_id`: The WSL session in which the command will be executed.
    /// - `program`: A Linux (UTF-8, Unix-style) path
    ///
    /// # Returns
    /// A [`WSLCommand`] builder ready to be configured and executed.
    ///
    /// # Notes
    ///
    /// - The default execution target is [`DistributionID::System`].
    /// - `argv[0]` defaults to the program path unless explicitly overridden.
    #[inline]
    pub fn new_command<'a, P: IntoCowUtf8UnixPath<'a>>(
        &'a self,
        session_id: SessionID,
        program: P,
    ) -> WSLCommand<'a> {
        WSLCommand::new(self, session_id, program)
    }

    fn check_required_version(&self, version: &WSLVersion) -> UpReqResult<()> {
        check_required_version_result(self.version(), version)
    }
}

impl Debug for ApiV1 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiV1")
            .field("version", self.version())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_transparence;

    #[test]
    fn test_layouts() {
        test_transparence::<WSLPluginAPIV1, ApiV1>();
    }
}
