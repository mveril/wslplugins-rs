#[cfg(doc)]
use super::Error;
use super::{Result, WSLCommand};
use crate::api::errors::require_update_error::Result as UpReqResult;
use crate::api::wsl_command::IntoCowUtf8UnixPath;
use crate::cstring_ext::CstringExt;
use crate::{SessionID, UserDistributionID, WSLVersion};
use std::ffi::{CString, OsStr};
use std::fmt::{self, Debug};
use std::iter::once;
use std::mem::MaybeUninit;
use std::net::TcpStream;
use std::os::windows::io::FromRawSocket as _;
use std::os::windows::raw::SOCKET;
use std::path::Path;
use std::ptr;
#[cfg(feature = "tracing")]
use tracing::instrument;
use typed_path::Utf8UnixPath;
use widestring::U16CString;
use windows_core::{Result as WinResult, HRESULT};
use wslpluginapi_sys;
use wslpluginapi_sys::windows_sys::Win32::Networking::WinSock::SOCKET as WinSocket;

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

    /// Execute a program in the root namespace.
    ///
    /// This method runs a program in the root namespace of the current WSL session. It connects the standard input and output
    /// streams of the executed process to a [`TcpStream`], allowing interaction with the process.
    ///
    /// # Arguments
    /// - `session`: The current WSL session.
    /// - `path`: Path to the program to execute.
    /// - `args`: Arguments to pass to the program (including `arg0`).
    ///
    /// # Returns
    /// On success, this method returns a [`TcpStream`] connected to the standard input and output streams of the executed process.
    /// - **Standard Input**: Data written to the stream will be sent to the process.
    /// - **Standard Output**: Data output by the process will be readable from the stream.
    ///
    /// # Errors
    /// This method can return the following a [`windows_core::Error`]: If the underlying Windows API call fails.
    ///
    /// # Example
    /// ```rust,ignore
    /// let stream = api.execute_binary(&session, "/bin/ls", ["/bin/ls", "-l", "/etc"])?;
    /// // Write to the process (stdin)
    /// writeln!(stream, "input data").unwrap();
    ///
    /// // Read from the process (stdout)
    /// let mut buffer = String::new();
    /// stream.read_to_string(&mut buffer).unwrap();
    /// println!("Process output: {}", buffer);
    /// ```
    #[cfg_attr(feature = "tracing", instrument(skip(args), level = "trace"))]
    #[doc(alias = "ExecuteBinary")]
    #[inline]
    pub fn execute_binary<P, I>(
        &self,
        session_id: SessionID,
        path: P,
        args: I,
    ) -> WinResult<TcpStream>
    where
        P: AsRef<Utf8UnixPath> + std::fmt::Debug,
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let c_path: Vec<u8> = path
            .as_ref()
            .as_str()
            .as_bytes()
            .iter()
            .copied()
            .chain(once(0))
            .collect();
        let c_args: Vec<CString> = args
            .into_iter()
            .map(|arg| CString::from_str_truncate(arg.as_ref()))
            .collect();
        let mut args_ptrs: Vec<*const u8> = c_args
            .iter()
            .map(|arg| arg.as_ptr().cast::<u8>())
            .chain(once(ptr::null::<u8>()))
            .collect();
        let args_ptr = args_ptrs.as_mut_ptr();
        let mut socket = MaybeUninit::<WinSocket>::uninit();
        // SAFETY:
        // - `ExecuteBinaryInDistribution` is guaranteed to be non-null because we first checked
        //   the API version (>= 2.1.2) before calling `unwrap_unchecked()`.
        // - `session.id()` returns a valid integer identifying a WSLSessionInformation`, which comes from
        //   a live session reference owned by the caller.
        // - `path_ptr` points to a null-terminated buffer (`c_path`) that is kept alive for the
        //   duration of the call.
        // - `args_ptr` points to a null-terminated array of pointers (`args_ptrs`), each pointing
        //   to a valid C string buffer (`c_args`). All these buffers live until after the call.
        // - `socket.as_mut_ptr()` is a valid, writable pointer to uninitialized memory. The
        //   function is documented to write a valid SOCKET there on success.
        // - We only call `assume_init()` if `HRESULT::ok()` reports success, ensuring the socket
        //   field has been properly initialized by the callee.
        // - `TcpStream::from_raw_socket` takes ownership of the returned socket; no double-close
        //   occurs because we never manually close it.
        let stream = unsafe {
            HRESULT(self.0.ExecuteBinary.unwrap_unchecked()(
                u32::from(session_id),
                c_path.as_ptr(),
                args_ptr,
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

    /// Execute a program in a user distribution
    ///
    /// # Introduced
    /// This requires API version 2.1.2 or later.
    ///
    /// # Arguments
    /// - `session`: The current WSL session.
    /// - `distribution_id`: The ID of the target distribution.
    /// - `path`: Path to the program to execute.
    /// - `args`: Arguments to pass to the program (including arg0).
    ///
    /// # Returns
    /// A [`TcpStream`] connected to the process's stdin and stdout.
    ///
    /// # Errors
    /// This function may return the following errors:
    ///
    /// - [`Error::RequiresUpdate`]: If the API version is lower than 2.1.2.
    /// - [`Error::WinError`]: If the Windows API fails during execution.
    ///
    /// # Example
    /// ```rust,ignore
    /// let stream = api.execute_binary_in_distribution(&session, "/bin/ls", ["/bin/ls", "-l", "/etc"])?;
    /// // Write to the process (stdin)
    /// writeln!(stream, "input data").unwrap();
    ///
    /// // Read from the process (stdout)
    /// let mut buffer = String::new();
    /// stream.read_to_string(&mut buffer).unwrap();
    /// println!("Process output: {}", buffer);
    /// ```
    #[doc(alias = "ExecuteBinaryInDistribution")]
    #[cfg_attr(feature = "tracing", instrument(skip(args), level = "trace"))]
    #[inline]
    pub fn execute_binary_in_distribution<P, I>(
        &self,
        session_id: SessionID,
        distribution_id: UserDistributionID,
        path: P,
        args: I,
    ) -> Result<TcpStream>
    where
        P: AsRef<Utf8UnixPath> + std::fmt::Debug,
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        self.check_required_version(&WSLVersion::new(2, 1, 2))?;
        let c_path: Vec<u8> = path
            .as_ref()
            .as_str()
            .as_bytes()
            .iter()
            .copied()
            .chain(once(0))
            .collect();
        let path_ptr = c_path.as_ptr();
        let c_args: Vec<CString> = args
            .into_iter()
            .map(|arg| CString::from_str_truncate(arg.as_ref()))
            .collect();
        let mut args_ptrs: Vec<_> = c_args
            .iter()
            .map(|arg| arg.as_ptr().cast::<u8>())
            .chain(once(ptr::null()))
            .collect();
        let args_ptr = args_ptrs.as_mut_ptr();
        let mut socket = MaybeUninit::<WinSocket>::uninit();
        let guid: wslpluginapi_sys::windows_sys::core::GUID = distribution_id.into();
        // SAFETY:
        // - `ExecuteBinaryInDistribution` is guaranteed to be non-null because we first checked
        //   the API version (>= 2.1.2) before calling `unwrap_unchecked()`.
        // - `session.id()` returns a valid integer identifying a WSLSessionInformation`, which comes from
        //   a live session reference owned by the caller.
        // - `distribution_id` is passed by reference as a properly aligned and sized GUID, cast
        //   to the expected FFI type.
        // - `path_ptr` points to a null-terminated buffer (`c_path`) that is kept alive for the
        //   duration of the call.
        // - `args_ptr` points to a null-terminated array of pointers (`args_ptrs`), each pointing
        //   to a valid C string buffer (`c_args`). All these buffers live until after the call.
        // - `socket.as_mut_ptr()` is a valid, writable pointer to uninitialized memory. The
        //   function is documented to write a valid SOCKET there on success.
        // - We only call `assume_init()` if `HRESULT::ok()` reports success, ensuring the socket
        //   field has been properly initialized by the callee.
        // - `TcpStream::from_raw_socket` takes ownership of the returned socket; no double-close
        //   occurs because we never manually close it.
        #[allow(clippy::absolute_paths)]
        let stream = unsafe {
            HRESULT(self.0.ExecuteBinaryInDistribution.unwrap_unchecked()(
                u32::from(session_id),
                (&raw const guid),
                path_ptr,
                args_ptr,
                socket.as_mut_ptr(),
            ))
            .ok()?;
            let socket = socket.assume_init();
            TcpStream::from_raw_socket(socket as SOCKET)
        };
        Ok(stream)
    }
    /// Creates a new `WSLCommand` instance tied to the current WSL API.
    ///
    /// This method initializes a `WSLCommand` with the provided session and
    /// program details. The program is specified as a path that can be converted
    /// to a `Utf8UnixPath`.
    ///
    /// # Parameters
    /// - `session`: The session information associated with the WSL instance.
    /// - `program`: A reference to the path of the program to be executed,
    ///   represented as an object implementing `AsRef<Utf8UnixPath>`.
    ///
    /// # Returns
    /// A new instance of `WSLCommand` configured to execute the specified program
    /// within the provided WSL session.
    ///
    /// # Type Parameters
    /// - `T`: A type that implements `AsRef<Utf8UnixPath>`.
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
