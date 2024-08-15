extern crate wslplugins_sys;
use std::ffi::{CString, OsStr, OsString};
use std::iter::once;
use std::mem::MaybeUninit;
use std::os::windows::io::FromRawSocket;
use windows::Win32::Networking::WinSock::SOCKET as WinSocket;
use std::os::windows::raw::SOCKET;
use std::net::TcpStream;
use std::path::Path;
use std::str::FromStr;
use crate::{wsl_version::WSLVersion, utils::{encode_wide_null_terminated, cstring_from_str}};
use windows::{core::{GUID, PCSTR, PCWSTR, Result}, Win32::Foundation::BOOL};
use crate::wsl_session_information::WSLSessionInformation;
use typed_path::Utf8UnixPath;
pub struct ApiV1(*const wslplugins_sys::WSLPluginAPIV1);

impl ApiV1 {
    pub fn from_raw(value: *const wslplugins_sys::WSLPluginAPIV1) -> Self {
        Self(value)
    }
    
    pub fn version(&self) -> WSLVersion {
        unsafe {
            let ver_ptr = &(*self.0).Version as *const wslplugins_sys::WSLVersion;
            WSLVersion::from_raw(ver_ptr)
        }
    }
    /// Create plan9 mount between Windows & Linux
    pub fn mount_folder<WP: AsRef<Path>, UP: AsRef<Utf8UnixPath>>(
        &self, session: &WSLSessionInformation, windows_path: WP, 
        linux_path: UP, read_only: bool, name: &OsStr
    ) -> Result<()> {
        let encoded_windows_path = encode_wide_null_terminated(windows_path.as_ref().as_os_str());
        let encoded_linux_path = encode_wide_null_terminated(OsString::from_str(linux_path.as_ref().as_str()).unwrap().as_os_str());
        let encoded_name = encode_wide_null_terminated(name);
        
        let result = unsafe {
            (*self.0).MountFolder.unwrap_unchecked()(
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
    pub fn execute_binary<P: AsRef<Utf8UnixPath>> (
        &self, session: &WSLSessionInformation, path: P, args: &[&str]
    ) -> Result<TcpStream> {
        let c_path: Vec<u8> = path.as_ref().as_str().as_bytes().iter().copied().chain(once(0)).collect();
        let c_args: Vec<CString> = args.iter().map(|&arg| cstring_from_str(arg)).collect();
        let mut args_ptrs: Vec<PCSTR> = c_args.iter().map(|arg| PCSTR::from_raw(arg.as_ptr() as *const u8)).chain(Some(PCSTR::null())).collect();
        let args_ptr = args_ptrs.as_mut_ptr();
        let mut socket = MaybeUninit::<WinSocket>::uninit();
        let stream = unsafe {
            (*self.0).ExecuteBinary.unwrap_unchecked()(
                session.id(), 
                PCSTR::from_raw(c_path.as_ptr()), 
                args_ptr,
                socket.as_mut_ptr()
            ).ok()?;
            let socket = socket.assume_init();
            TcpStream::from_raw_socket(socket.0 as SOCKET)
        };
        Ok(stream)
    }

    // Set the error message to display to the user if the VM or distribution creation fails.
    pub fn plugin_error(&self, error: &OsStr) -> Result<()> {
        let error_vec = encode_wide_null_terminated(error);
        unsafe {
            (*self.0).PluginError.unwrap_unchecked()(PCWSTR::from_raw(error_vec.as_ptr())).ok()
        }
    }
    /// Execute a program in a user distribution
    /// Introduced in 2.1.2
    pub fn execute_binary_in_distribution<P: AsRef<Utf8UnixPath>>(
        &self, session: &WSLSessionInformation, distribution_id: GUID, path: P, args: &[&str]
    ) -> Result<TcpStream> {
        let c_path: Vec<u8> = path.as_ref().as_str().as_bytes().iter().copied().chain(once(0)).collect();
        let path_ptr = PCSTR::from_raw(c_path.as_ptr());
        let c_args: Vec<CString> = args.iter().map(|&arg| cstring_from_str(arg)).collect();
        let mut args_ptrs: Vec<PCSTR> = c_args.iter().map(|arg| PCSTR::from_raw(arg.as_ptr() as *const u8)).chain(Some(PCSTR::null())).collect();
        let args_ptr = args_ptrs.as_mut_ptr();
        let mut socket = MaybeUninit::<WinSocket>::uninit();
        let stream = unsafe {
            (*self.0).ExecuteBinaryInDistribution.unwrap_unchecked()(session.id(), &distribution_id, path_ptr, args_ptr, socket.as_mut_ptr()).ok()?;
            let socket = socket.assume_init();
            TcpStream::from_raw_socket(socket.0 as SOCKET)
        };
        Ok(stream)
    }
}
