extern crate wslplugins_sys;
#[cfg(doc)]
use super::Error;
use super::Result;
use crate::api::errors::require_update_error::Result as UpReqResult;
use crate::utils::{cstring_from_str, encode_wide_null_terminated};
use crate::wsl_session_information::WSLSessionInformation;
#[cfg(feature = "log-instrument")]
use log_instrument::instrument;
use std::ffi::{CString, OsStr, OsString};
use std::fmt::Debug;
use std::iter::once;
use std::mem::MaybeUninit;
use std::net::TcpStream;
use std::os::windows::io::FromRawSocket;
use std::os::windows::raw::SOCKET;
use std::path::Path;
use std::str::FromStr;
use typed_path::Utf8UnixPath;
use windows::Win32::Networking::WinSock::SOCKET as WinSocket;
use windows::{
    core::{Result as WinResult, GUID, PCSTR, PCWSTR},
    Win32::Foundation::BOOL,
};

use wslplugins_sys::{WSLPluginAPIV1, WSLVersion};

use super::utils::check_required_version_result;

/// Represents a structured interface for interacting with the WSLPluginAPIV1 API.
/// This struct encapsulates the methods provided by the WSLPluginAPIV1 API, allowing
/// idiomatic interaction with the Windows Subsystem for Linux (WSL).
pub struct ApiV1<'a>(&'a WSLPluginAPIV1);

/// Converts a raw reference to `WSLPluginAPIV1` into [ApiV1].
impl<'a> From<&'a WSLPluginAPIV1> for ApiV1<'a> {
    fn from(value: &'a WSLPluginAPIV1) -> Self {
        Self(value)
    }
}

impl ApiV1<'_> {
    /// Returns the current version of the WSL API being used.
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
    #[cfg_attr(feature = "log-instrument", instrument)]
    pub fn version(&self) -> &WSLVersion {
        &self.0.Version
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
    ///
    /// # Example
    /// ``` rust,ignore
    /// api.mount_folder(&session, "C:\\path", "/mnt/path", false, "MyMount")?;
    /// ```
    #[doc(alias = "MountFolder")]
    #[cfg_attr(feature = "log-instrument", instrument)]
    pub fn mount_folder<WP: AsRef<Path>, UP: AsRef<Utf8UnixPath>>(
        &self,
        session: &WSLSessionInformation,
        windows_path: WP,
        linux_path: UP,
        read_only: bool,
        name: &OsStr,
    ) -> WinResult<()> {
        let encoded_windows_path = encode_wide_null_terminated(windows_path.as_ref().as_os_str());
        let encoded_linux_path = encode_wide_null_terminated(
            OsString::from_str(linux_path.as_ref().as_str())
                .unwrap()
                .as_os_str(),
        );
        let encoded_name = encode_wide_null_terminated(name);
        let result = unsafe {
            self.0.MountFolder.unwrap_unchecked()(
                session.id(),
                PCWSTR::from_raw(encoded_windows_path.as_ptr()),
                PCWSTR::from_raw(encoded_linux_path.as_ptr()),
                BOOL::from(read_only),
                PCWSTR::from_raw(encoded_name.as_ptr()),
            )
        };
        result.ok()
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
    /// This method can return the following a [`windows::core::Error`]: If the underlying Windows API call fails.
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
    #[cfg_attr(feature = "log-instrument", instrument)]
    #[doc(alias = "ExecuteBinary")]
    pub fn execute_binary<P: AsRef<Utf8UnixPath>>(
        &self,
        session: &WSLSessionInformation,
        path: P,
        args: &[&str],
    ) -> WinResult<TcpStream> {
        let c_path: Vec<u8> = path
            .as_ref()
            .as_str()
            .as_bytes()
            .iter()
            .copied()
            .chain(once(0))
            .collect();
        let c_args: Vec<CString> = args.iter().map(|&arg| cstring_from_str(arg)).collect();
        let mut args_ptrs: Vec<PCSTR> = c_args
            .iter()
            .map(|arg| PCSTR::from_raw(arg.as_ptr() as *const u8))
            .chain(Some(PCSTR::null()))
            .collect();
        let args_ptr = args_ptrs.as_mut_ptr();
        let mut socket = MaybeUninit::<WinSocket>::uninit();
        let stream = unsafe {
            self.0.ExecuteBinary.unwrap_unchecked()(
                session.id(),
                PCSTR::from_raw(c_path.as_ptr()),
                args_ptr,
                socket.as_mut_ptr(),
            )
            .ok()?;
            let socket = socket.assume_init();
            TcpStream::from_raw_socket(socket.0 as SOCKET)
        };
        Ok(stream)
    }

    /// Set the error message to display to the user if the VM or distribution creation fails.
    #[cfg_attr(feature = "log-instrument", instrument)]
    pub(crate) fn plugin_error(&self, error: &OsStr) -> WinResult<()> {
        let error_vec = encode_wide_null_terminated(error);
        unsafe { self.0.PluginError.unwrap_unchecked()(PCWSTR::from_raw(error_vec.as_ptr())).ok() }
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
    #[cfg_attr(feature = "log-instrument", instrument)]
    pub fn execute_binary_in_distribution<P: AsRef<Utf8UnixPath>>(
        &self,
        session: &WSLSessionInformation,
        distribution_id: &GUID,
        path: P,
        args: &[&str],
    ) -> Result<TcpStream> {
        self.check_required_version(&WSLVersion::new(2, 1, 2))?;
        let c_path: Vec<u8> = path
            .as_ref()
            .as_str()
            .as_bytes()
            .iter()
            .copied()
            .chain(once(0))
            .collect();
        let path_ptr = PCSTR::from_raw(c_path.as_ptr());
        let c_args: Vec<CString> = args.iter().map(|&arg| cstring_from_str(arg)).collect();
        let mut args_ptrs: Vec<PCSTR> = c_args
            .iter()
            .map(|arg| PCSTR::from_raw(arg.as_ptr() as *const u8))
            .chain(Some(PCSTR::null()))
            .collect();
        let args_ptr = args_ptrs.as_mut_ptr();
        let mut socket = MaybeUninit::<WinSocket>::uninit();
        let stream = unsafe {
            self.0.ExecuteBinaryInDistribution.unwrap_unchecked()(
                session.id(),
                distribution_id,
                path_ptr,
                args_ptr,
                socket.as_mut_ptr(),
            )
            .ok()?;
            let socket = socket.assume_init();
            TcpStream::from_raw_socket(socket.0 as SOCKET)
        };
        Ok(stream)
    }

    fn check_required_version(&self, version: &WSLVersion) -> UpReqResult<()> {
        check_required_version_result(self.version(), version)
    }
}

impl Debug for ApiV1<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiV1")
            .field("version", self.version())
            .finish()
    }
}
