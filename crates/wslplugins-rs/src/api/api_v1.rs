#[cfg(doc)]
use super::Error;
use super::{Result, WSLCommand};
use crate::api::errors::require_update_error::Result as UpReqResult;
use crate::api::wsl_command::IntoCowUtf8UnixPath;
use crate::{HasSessionId, SessionID, UserDistributionID, WSLVersion, WSLVersionCapability};
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

use super::utils::check_requirement_result;

/// Represents a structured interface for interacting with the `WSLPluginAPIV1` API.
///
/// This struct encapsulates the methods provided by the `WSLPluginAPIV1` API, allowing
/// idiomatic interaction with the Windows Subsystem for Linux (WSL).
// Only fields supported by the host version are accessed through this view.
#[repr(C)]
pub struct ApiV1 {
    _private: (),
    _marker: std::marker::PhantomData<std::marker::PhantomPinned>,
}

impl ApiV1 {
    /// Borrows a versioned WSL structure without referencing its full current layout.
    ///
    /// # Safety
    /// `raw` must be non-null, correctly aligned, and point to an immutable WSL
    /// structure containing every field supported by the current host version.
    /// The allocation must remain valid for `'a`; an API installed in the global
    /// context must remain valid for the plugin lifetime. The host must support
    /// shared API calls, since this immutable view is Send and Sync.
    #[inline]
    #[must_use]
    pub const unsafe fn from_raw<'a>(raw: *const wslpluginapi_sys::WSLPluginAPIV1) -> &'a Self {
        // SAFETY: The caller guarantees the allocation for this opaque borrowed view.
        unsafe { &*raw.cast::<Self>() }
    }

    /// Returns the pointer used for field-by-field ABI access.
    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const wslpluginapi_sys::WSLPluginAPIV1 {
        std::ptr::from_ref(self).cast()
    }
}

impl ApiV1 {
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
    ///     version.major(),
    ///     version.minor(),
    ///     version.revision(),
    /// );
    #[must_use]
    #[inline]
    pub const fn version(&self) -> &WSLVersion {
        // SAFETY: Version is in the baseline ABI and from_raw requires aligned storage.
        unsafe { &*std::ptr::addr_of!((*self.as_ptr()).Version).cast::<WSLVersion>() }
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
        S: AsRef<OsStr> + std::fmt::Debug,
    >(
        &self,
        session_id: SessionID,
        windows_path: WP,
        linux_path: UP,
        read_only: bool,
        name: S,
    ) -> WinResult<()> {
        let encoded_windows_path =
            U16CString::from_os_str_truncate(windows_path.as_ref().as_os_str());
        let encoded_linux_path = U16CString::from_str_truncate(linux_path.as_ref().as_str());
        let encoded_name = U16CString::from_os_str_truncate(name.as_ref());
        // SAFETY:
        // - MountFolder belongs to the baseline WSL API layout.
        //   A missing function pointer is reported as E_NOTIMPL.
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
            std::ptr::addr_of!((*self.as_ptr()).MountFolder)
                .read_unaligned()
                .ok_or_else(|| {
                    windows_core::Error::from_hresult(HRESULT(
                        wslpluginapi_sys::windows_sys::Win32::Foundation::E_NOTIMPL,
                    ))
                })?(
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
            HRESULT(std::ptr::addr_of!((*self.as_ptr()).ExecuteBinary)
                .read_unaligned()
                .ok_or_else(|| {
                    windows_core::Error::from_hresult(HRESULT(
                        wslpluginapi_sys::windows_sys::Win32::Foundation::E_NOTIMPL,
                    ))
                })?(
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
            unsafe {
                std::ptr::addr_of!((*self.as_ptr()).PluginError)
                    .read_unaligned()
                    .ok_or_else(|| {
                        windows_core::Error::from_hresult(HRESULT(
                            wslpluginapi_sys::windows_sys::Win32::Foundation::E_NOTIMPL,
                        ))
                    })?(error_utf16.as_ptr())
            },
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
        self.require_capability(WSLVersionCapability::ExecuteBinaryInDistribution)?;
        let mut socket = MaybeUninit::<WinSocket>::uninit();
        let guid: wslpluginapi_sys::windows_sys::core::GUID = distribution_id.into();
        // SAFETY: Calling ExecuteBinaryInDistribution is safe with agument correctly prepared.
        let stream = unsafe {
            HRESULT(std::ptr::addr_of!(
                (*self.as_ptr()).ExecuteBinaryInDistribution
            )
            .read_unaligned()
            .ok_or_else(|| {
                windows_core::Error::from_hresult(HRESULT(
                    wslpluginapi_sys::windows_sys::Win32::Foundation::E_NOTIMPL,
                ))
            })?(
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
    pub fn new_command<'a, S: HasSessionId, P: IntoCowUtf8UnixPath<'a>>(
        &'a self,
        session_id: S,
        program: P,
    ) -> WSLCommand<'a> {
        WSLCommand::new(self, session_id, program)
    }

    fn require_capability(&self, capability: WSLVersionCapability) -> UpReqResult<()> {
        check_requirement_result(self.version(), capability)
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

    // Original API table, before ExecuteBinaryInDistribution was introduced.
    #[repr(C)]
    struct BaselineApi {
        version: wslpluginapi_sys::WSLVersion,
        mount_folder: wslpluginapi_sys::WSLPluginAPI_MountFolder,
        execute_binary: wslpluginapi_sys::WSLPluginAPI_ExecuteBinary,
        plugin_error: wslpluginapi_sys::WSLPluginAPI_PluginError,
    }

    #[test]
    fn reads_api_version_without_newer_function_slots() {
        let raw = BaselineApi {
            version: WSLVersion::new(2, 0, 0).into(),
            mount_folder: None,
            execute_binary: None,
            plugin_error: None,
        };
        // SAFETY: The immutable original API table remains alive for this borrow.
        let api = unsafe { ApiV1::from_raw(std::ptr::from_ref(&raw).cast()) };
        assert_eq!(*api.version(), WSLVersion::new(2, 0, 0));
        assert!(api
            .require_capability(WSLVersionCapability::ExecuteBinaryInDistribution)
            .is_err());
        assert!(api.plugin_error(OsStr::new("test")).is_err());
    }

    #[test]
    fn opaque_view_roundtrips_native_pointer_and_reads_native_field() {
        let native = BaselineApi {
            version: WSLVersion::new(2, 0, 0).into(),
            mount_folder: None,
            execute_binary: None,
            plugin_error: None,
        };
        let native_ptr: *const BaselineApi = std::ptr::from_ref(&native);
        let api_ptr: *const wslpluginapi_sys::WSLPluginAPIV1 = native_ptr.cast();

        // SAFETY: BaselineApi reproduces the complete API prefix represented by this host version.
        let opaque: &ApiV1 = unsafe { ApiV1::from_raw(api_ptr) };
        let roundtripped_ptr: *const BaselineApi = opaque.as_ptr().cast();
        // SAFETY: The pointer round-trips to the still-live native BaselineApi value.
        let observed_version =
            unsafe { std::ptr::addr_of!((*roundtripped_ptr).version).read_unaligned() };

        assert_eq!(roundtripped_ptr, native_ptr);
        assert_eq!(observed_version, native.version);
    }
}
