use super::process::WSLCProcessInner;
use super::{
    WSLCChildStderr, WSLCChildStdin, WSLCChildStdout, WSLCCommand, WSLCCreateProcessError,
    WSLCProcess, WSLCProcessFd, WSLCProcessStreams, WSLCSessionID, WSLCSpawnError,
};
use crate::api::wsl_command::IntoCowUtf8UnixPath;
use crate::api::{ApiV1, Result};
use crate::cstring_ext::CstringExt as _;
use std::ffi::{c_void, CString};
use std::fmt;
use std::mem::MaybeUninit;
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::path::Path;
use std::ptr::{self, NonNull};
use std::sync::Arc;
use typed_path::Utf8UnixPath;
use widestring::U16CString;
use windows_core::{Error as WinError, HRESULT};
use wslpluginapi_sys::WSLCProcessHandle;
const E_POINTER: HRESULT = HRESULT(0x8000_4003_u32.cast_signed());
///
/// Obtain this view through [`ApiV1::wslc`]. Constructing the view performs
/// the `2.9.0` capability check once; its methods can then use the WSLC
/// function table without repeating that check.
#[derive(Clone, Copy)]
pub struct WSLCApi<'a> {
    api: &'a ApiV1,
}

impl<'a> WSLCApi<'a> {
    #[inline]
    pub(crate) const fn new(api: &'a ApiV1) -> Self {
        Self { api }
    }

    /// Mounts a Windows folder in a WSLC session.
    ///
    /// # Errors
    ///fprocree
    /// Returns an error when WSL rejects the mount operation.
    #[doc(alias = "WSLCMountFolder")]
    #[inline]
    pub fn mount_folder<WP, MP>(
        &self,
        session_id: WSLCSessionID,
        windows_path: WP,
        mountpoint: MP,
        read_only: bool,
    ) -> Result<()>
    where
        WP: AsRef<Path>,
        MP: AsRef<Utf8UnixPath>,
    {
        let windows_path = U16CString::from_os_str_truncate(windows_path.as_ref().as_os_str());
        let mountpoint = CString::from_str_truncate(mountpoint.as_ref().as_str());
        let raw = self.raw();

        // SAFETY: `WSLCApi` can only be obtained for a host version that
        // supplies this function. Both strings are NUL-terminated and remain
        // alive for the duration of the call.
        unsafe {
            HRESULT(raw.WSLCMountFolder.unwrap_unchecked()(
                session_id.get(),
                windows_path.as_ptr(),
                mountpoint.as_ptr().cast(),
                i32::from(read_only),
            ))
            .ok()
            .map_err(Into::into)
        }
    }

    /// Unmounts a folder previously mounted through [`Self::mount_folder`].
    ///
    /// # Errors
    ///
    /// Returns an error when WSL rejects the unmount operation.
    #[doc(alias = "WSLCUnmountFolder")]
    #[inline]
    pub fn unmount_folder<MP>(&self, session_id: WSLCSessionID, mountpoint: MP) -> Result<()>
    where
        MP: AsRef<Utf8UnixPath>,
    {
        let mountpoint = CString::from_str_truncate(mountpoint.as_ref().as_str());
        let raw = self.raw();

        // SAFETY: `WSLCApi` guarantees function availability and `mountpoint`
        // is a live, NUL-terminated C string.
        unsafe {
            HRESULT(raw.WSLCUnmountFolder.unwrap_unchecked()(
                session_id.get(),
                mountpoint.as_ptr().cast(),
            ))
            .ok()
            .map_err(Into::into)
        }
    }

    /// Creates a command builder for the root namespace of a WSLC session.
    #[must_use]
    #[inline]
    pub fn new_command<'command, P>(
        &self,
        session_id: WSLCSessionID,
        program: P,
    ) -> WSLCCommand<'a, 'command>
    where
        P: IntoCowUtf8UnixPath<'command>,
    {
        WSLCCommand::new(*self, session_id, program)
    }

    #[inline]
    pub(super) fn raw(self) -> &'a wslpluginapi_sys::WSLPluginAPIV1 {
        self.api.as_ref()
    }

    pub(super) fn create_process_encoded(
        self,
        session_id: WSLCSessionID,
        executable: &[u8],
        argv: &[*const u8],
        environment: &[*const u8],
        streams: WSLCProcessStreams,
    ) -> std::result::Result<WSLCProcess<'a>, WSLCSpawnError> {
        let mut process = MaybeUninit::<WSLCProcessHandle>::uninit();
        let mut errno = 0;
        let raw = self.raw();

        // SAFETY: all strings and pointer arrays are NUL-terminated and remain
        // alive during the call. The output slots are valid for writes.
        let result = unsafe {
            HRESULT(raw.WSLCCreateProcess.unwrap_unchecked()(
                session_id.get(),
                executable.as_ptr().cast(),
                argv.as_ptr().cast_mut().cast(),
                environment.as_ptr().cast_mut().cast(),
                process.as_mut_ptr(),
                ptr::from_mut(&mut errno),
            ))
        };

        if let Err(source) = result.ok() {
            return Err(WSLCCreateProcessError::new(source, errno).into());
        }

        // SAFETY: a successful WSLC create-process call initializes `process`.
        let process = unsafe { process.assume_init() };
        let handle = NonNull::new(process.cast())
            .ok_or_else(|| WSLCCreateProcessError::new(WinError::from(E_POINTER), errno))?;
        let process = Arc::new(WSLCProcessInner { api: self, handle });
        let stdin = streams
            .contains(WSLCProcessStreams::STDIN)
            .then(|| self.process_handle(handle, WSLCProcessFd::Stdin))
            .transpose()?;
        let stdout = streams
            .contains(WSLCProcessStreams::STDOUT)
            .then(|| self.process_handle(handle, WSLCProcessFd::Stdout))
            .transpose()?;
        let stderr = streams
            .contains(WSLCProcessStreams::STDERR)
            .then(|| self.process_handle(handle, WSLCProcessFd::Stderr))
            .transpose()?;
        let exit_event = self.process_exit_event(handle)?;

        Ok(WSLCProcess {
            inner: Arc::clone(&process),
            stdin: stdin.map(|handle| WSLCChildStdin::new(handle, Arc::clone(&process))),
            stdout: stdout.map(|handle| WSLCChildStdout::new(handle, Arc::clone(&process))),
            stderr: stderr.map(|handle| WSLCChildStderr::new(handle, process)),
            exit_event,
            exit_status: None,
        })
    }

    pub(super) fn process_handle(
        self,
        process: NonNull<c_void>,
        fd: WSLCProcessFd,
    ) -> Result<OwnedHandle> {
        let mut handle = MaybeUninit::uninit();
        let raw = self.raw();

        // SAFETY: `process` is owned by a live `WSLCProcess`; the output slot
        // is valid. On success WSL transfers ownership of the handle.
        unsafe {
            HRESULT(raw.WSLCProcessGetFd.unwrap_unchecked()(
                process.as_ptr(),
                fd as i32,
                handle.as_mut_ptr(),
            ))
            .ok()?;

            let handle = handle.assume_init();
            if handle.is_null() {
                return Err(WinError::from(E_POINTER).into());
            }
            Ok(OwnedHandle::from_raw_handle(handle.cast()))
        }
    }

    pub(super) fn process_exit_event(self, process: NonNull<c_void>) -> Result<OwnedHandle> {
        let mut handle = MaybeUninit::uninit();
        let raw = self.raw();

        // SAFETY: `process` is owned by a live `WSLCProcess`; the output slot
        // is valid. On success WSL transfers ownership of the event handle.
        unsafe {
            HRESULT(raw.WSLCProcessGetExitEvent.unwrap_unchecked()(
                process.as_ptr(),
                handle.as_mut_ptr(),
            ))
            .ok()?;

            let handle = handle.assume_init();
            if handle.is_null() {
                return Err(WinError::from(E_POINTER).into());
            }
            Ok(OwnedHandle::from_raw_handle(handle.cast()))
        }
    }

    pub(super) fn process_exit_code(self, process: NonNull<c_void>) -> Result<i32> {
        let mut exit_code = MaybeUninit::uninit();
        let raw = self.raw();

        // SAFETY: `process` is owned by a live `WSLCProcess` and the output
        // slot is valid. The caller must wait for process exit first.
        unsafe {
            HRESULT(raw.WSLCProcessGetExitCode.unwrap_unchecked()(
                process.as_ptr(),
                exit_code.as_mut_ptr(),
            ))
            .ok()?;
            Ok(exit_code.assume_init())
        }
    }
}

impl fmt::Debug for WSLCApi<'_> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WSLCApi")
            .field("version", self.api.version())
            .finish()
    }
}
