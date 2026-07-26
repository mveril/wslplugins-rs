use super::wsl_command::IntoCowUtf8UnixPath;
use super::{utils, ApiV1, Result};
use crate::cstring_ext::CstringExt as _;
use std::borrow::Cow;
use std::ffi::{c_void, CString};
use std::fmt;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::os::windows::io::{
    AsHandle, AsRawHandle, BorrowedHandle, FromRawHandle, OwnedHandle, RawHandle,
};
use std::path::Path;
use std::ptr::{self, NonNull};
use thiserror::Error;
use typed_path::Utf8UnixPath;
use widestring::U16CString;
use windows_core::{Error as WinError, HRESULT};
use wslpluginapi_sys::WSLCProcessHandle;

const E_POINTER: HRESULT = HRESULT(0x8000_4003_u32.cast_signed());

/// Identifies a WSL container session.
///
/// WSLC session identifiers are a different identifier space from regular WSL
/// session identifiers, so this type deliberately does not convert to or from
/// [`crate::SessionID`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct WSLCSessionID(u32);

impl WSLCSessionID {
    /// Creates an identifier from its native value.
    #[must_use]
    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the native identifier value.
    #[must_use]
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for WSLCSessionID {
    #[inline]
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl From<WSLCSessionID> for u32 {
    #[inline]
    fn from(value: WSLCSessionID) -> Self {
        value.get()
    }
}

/// The standard stream requested from a WSLC process.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum WSLCProcessFd {
    /// Standard input.
    Stdin = wslpluginapi_sys::WSLCProcessFd_WSLCProcessFdStdin,
    /// Standard output.
    Stdout = wslpluginapi_sys::WSLCProcessFd_WSLCProcessFdStdout,
    /// Standard error.
    Stderr = wslpluginapi_sys::WSLCProcessFd_WSLCProcessFdStderr,
}

/// API operations scoped to WSL container sessions.
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
    pub(super) const fn new(api: &'a ApiV1) -> Self {
        Self { api }
    }

    /// Mounts a Windows folder in a WSLC session.
    ///
    /// # Errors
    ///
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
    fn raw(self) -> &'a wslpluginapi_sys::WSLPluginAPIV1 {
        self.api.as_ref()
    }

    fn create_process(
        self,
        session_id: WSLCSessionID,
        executable: &Utf8UnixPath,
        argv: impl IntoIterator<Item = impl AsRef<str>>,
        environment: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> std::result::Result<WSLCProcess<'a>, WSLCCreateProcessError> {
        let executable = utils::encode_c_path(executable);
        let (_arguments, argv) = utils::encode_c_argv(argv);
        let (_environment, environment) = utils::encode_c_argv(environment);
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
            return Err(WSLCCreateProcessError::new(source, errno));
        }

        // SAFETY: a successful WSLC create-process call initializes `process`.
        let process = unsafe { process.assume_init() };
        let handle = NonNull::new(process.cast())
            .ok_or_else(|| WSLCCreateProcessError::new(WinError::from(E_POINTER), errno))?;

        Ok(WSLCProcess { api: self, handle })
    }

    fn process_handle(self, process: NonNull<c_void>, fd: WSLCProcessFd) -> Result<OwnedHandle> {
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

    fn process_exit_event(self, process: NonNull<c_void>) -> Result<OwnedHandle> {
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

    fn process_exit_code(self, process: NonNull<c_void>) -> Result<i32> {
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

/// A command to create a process in a WSLC session's root namespace.
#[derive(Clone, Debug)]
pub struct WSLCCommand<'api, 'command> {
    api: WSLCApi<'api>,
    session_id: WSLCSessionID,
    program: Cow<'command, Utf8UnixPath>,
    arg0: Option<Cow<'command, str>>,
    args: Vec<Cow<'command, str>>,
    environment: Vec<Cow<'command, str>>,
}

impl<'api, 'command> WSLCCommand<'api, 'command> {
    fn new<P>(api: WSLCApi<'api>, session_id: WSLCSessionID, program: P) -> Self
    where
        P: IntoCowUtf8UnixPath<'command>,
    {
        Self {
            api,
            session_id,
            program: program.into_cow_utf8_unix_path(),
            arg0: None,
            args: Vec::new(),
            environment: Vec::new(),
        }
    }

    /// Sets `argv[0]`.
    #[must_use]
    #[inline]
    pub fn with_arg0<T>(mut self, arg0: T) -> Self
    where
        T: Into<Cow<'command, str>>,
    {
        self.arg0 = Some(arg0.into());
        self
    }

    /// Adds one command-line argument.
    #[must_use]
    #[inline]
    pub fn with_arg<T>(mut self, argument: T) -> Self
    where
        T: Into<Cow<'command, str>>,
    {
        self.args.push(argument.into());
        self
    }

    /// Adds command-line arguments.
    #[must_use]
    #[inline]
    pub fn with_args<I, T>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Cow<'command, str>>,
    {
        self.args.extend(arguments.into_iter().map(Into::into));
        self
    }

    /// Adds an environment entry in `NAME=VALUE` form.
    #[must_use]
    #[inline]
    pub fn with_environment<T>(mut self, entry: T) -> Self
    where
        T: Into<Cow<'command, str>>,
    {
        self.environment.push(entry.into());
        self
    }

    /// Creates the process.
    ///
    /// # Errors
    ///
    /// Returns the Windows failure and the Linux `errno` reported by WSL when
    /// process creation fails.
    #[doc(alias = "WSLCCreateProcess")]
    #[inline]
    pub fn spawn(&self) -> std::result::Result<WSLCProcess<'api>, WSLCCreateProcessError> {
        let arg0 = self.arg0.as_deref().unwrap_or(self.program.as_str());
        let arguments = std::iter::once(arg0).chain(self.args.iter().map(AsRef::as_ref));
        self.api.create_process(
            self.session_id,
            &self.program,
            arguments,
            self.environment.iter().map(AsRef::as_ref),
        )
    }
}

/// Failure returned while creating a process in a WSLC session.
#[derive(Clone, Debug, Error)]
#[error("failed to create WSLC process (errno {errno}): {source}")]
pub struct WSLCCreateProcessError {
    source: WinError,
    errno: i32,
}

impl WSLCCreateProcessError {
    const fn new(source: WinError, errno: i32) -> Self {
        Self { source, errno }
    }

    /// Returns the Linux error number reported by WSLC.
    #[must_use]
    #[inline]
    pub const fn errno(&self) -> i32 {
        self.errno
    }

    /// Returns the Windows API error.
    #[must_use]
    #[inline]
    pub const fn windows_error(&self) -> &WinError {
        &self.source
    }
}

/// An owned process created in a WSLC session.
///
/// Dropping this value releases the native process object. Handles returned by
/// [`Self::stdin`], [`Self::stdout`], [`Self::stderr`] and
/// [`Self::exit_event`] borrow the process, ensuring they are closed first.
pub struct WSLCProcess<'api> {
    api: WSLCApi<'api>,
    handle: NonNull<c_void>,
}

impl WSLCProcess<'_> {
    /// Returns an owned handle connected to standard input.
    ///
    /// # Errors
    ///
    /// Returns an error when WSL cannot provide the handle.
    #[inline]
    pub fn stdin(&self) -> Result<WSLCProcessOwnedHandle<'_>> {
        self.fd(WSLCProcessFd::Stdin)
    }

    /// Returns an owned handle connected to standard output.
    ///
    /// # Errors
    ///
    /// Returns an error when WSL cannot provide the handle.
    #[inline]
    pub fn stdout(&self) -> Result<WSLCProcessOwnedHandle<'_>> {
        self.fd(WSLCProcessFd::Stdout)
    }

    /// Returns an owned handle connected to standard error.
    ///
    /// # Errors
    ///
    /// Returns an error when WSL cannot provide the handle.
    #[inline]
    pub fn stderr(&self) -> Result<WSLCProcessOwnedHandle<'_>> {
        self.fd(WSLCProcessFd::Stderr)
    }

    /// Returns the event signaled when this process exits.
    ///
    /// # Errors
    ///
    /// Returns an error when WSL cannot provide the event handle.
    #[inline]
    pub fn exit_event(&self) -> Result<WSLCProcessOwnedHandle<'_>> {
        self.api
            .process_exit_event(self.handle)
            .map(WSLCProcessOwnedHandle::new)
    }

    /// Returns the process exit code.
    ///
    /// Call this only after the event returned by [`Self::exit_event`] has
    /// signaled.
    ///
    /// # Errors
    ///
    /// Returns an error if the process is still running or WSL cannot retrieve
    /// its exit code.
    #[inline]
    pub fn exit_code(&self) -> Result<i32> {
        self.api.process_exit_code(self.handle)
    }

    fn fd(&self, fd: WSLCProcessFd) -> Result<WSLCProcessOwnedHandle<'_>> {
        self.api
            .process_handle(self.handle, fd)
            .map(WSLCProcessOwnedHandle::new)
    }
}

impl fmt::Debug for WSLCProcess<'_> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WSLCProcess")
            .field("handle", &self.handle)
            .finish_non_exhaustive()
    }
}

impl Drop for WSLCProcess<'_> {
    #[inline]
    fn drop(&mut self) {
        let raw = self.api.raw();
        // SAFETY: this object uniquely owns the process handle and every
        // process-derived handle must be dropped before this borrow can end.
        unsafe {
            raw.WSLCReleaseProcess.unwrap_unchecked()(self.handle.as_ptr());
        }
    }
}

/// A Windows handle owned by a WSLC process operation.
///
/// Its lifetime prevents the parent [`WSLCProcess`] from being released before
/// this handle has been closed.
pub struct WSLCProcessOwnedHandle<'process> {
    handle: OwnedHandle,
    _process: PhantomData<&'process WSLCProcess<'process>>,
}

impl WSLCProcessOwnedHandle<'_> {
    const fn new(handle: OwnedHandle) -> Self {
        Self {
            handle,
            _process: PhantomData,
        }
    }
}

impl AsHandle for WSLCProcessOwnedHandle<'_> {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        self.handle.as_handle()
    }
}

impl AsRawHandle for WSLCProcessOwnedHandle<'_> {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.handle.as_raw_handle()
    }
}

impl fmt::Debug for WSLCProcessOwnedHandle<'_> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.handle.fmt(formatter)
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::missing_panics_doc,
    clippy::unwrap_used,
    reason = "Test synchronization and FFI assertions"
)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex;
    use widestring::U16CStr;
    use wslpluginapi_sys::windows_sys::Win32::Foundation::{E_FAIL, HANDLE, S_OK};

    #[derive(Debug, Default, PartialEq, Eq)]
    struct MountCall {
        session_id: u32,
        windows_path: String,
        mountpoint: String,
        read_only: bool,
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    struct ProcessCall {
        session_id: u32,
        executable: String,
        argv: Vec<String>,
        environment: Vec<String>,
    }

    static MOUNT_CALL: Mutex<Option<MountCall>> = Mutex::new(None);
    static UNMOUNT_CALL: Mutex<Option<(u32, String)>> = Mutex::new(None);
    static PROCESS_CALL: Mutex<Option<ProcessCall>> = Mutex::new(None);
    static FD_CALLS: Mutex<Vec<i32>> = Mutex::new(Vec::new());
    static TEST_LOCK: Mutex<()> = Mutex::new(());
    static CREATE_PROCESS_FAILS: AtomicBool = AtomicBool::new(false);
    static RELEASE_COUNT: AtomicUsize = AtomicUsize::new(0);

    unsafe fn read_c_string(pointer: *const u8) -> String {
        // SAFETY: test callbacks receive NUL-terminated strings from the wrapper.
        unsafe { CStr::from_ptr(pointer.cast()) }
            .to_string_lossy()
            .into_owned()
    }

    unsafe fn read_string_array(mut pointer: *mut *const u8) -> Vec<String> {
        let mut values = Vec::new();
        // SAFETY: the wrapper passes a NUL-terminated pointer array.
        while !unsafe { *pointer }.is_null() {
            // SAFETY: every non-null entry points to a live C string.
            values.push(unsafe { read_c_string(*pointer) });
            // SAFETY: the terminator guarantees that advancing remains in the array.
            pointer = unsafe { pointer.add(1) };
        }
        values
    }

    unsafe extern "C" fn mount_folder(
        session: u32,
        windows_path: *const u16,
        mountpoint: *const u8,
        read_only: i32,
    ) -> i32 {
        // SAFETY: the wrapper passes live NUL-terminated strings.
        let windows_path = unsafe { U16CStr::from_ptr_str(windows_path) }.to_string_lossy();
        // SAFETY: the wrapper passes a live NUL-terminated string.
        let mountpoint = unsafe { read_c_string(mountpoint) };
        *MOUNT_CALL.lock().unwrap() = Some(MountCall {
            session_id: session,
            windows_path,
            mountpoint,
            read_only: read_only != 0,
        });
        S_OK
    }

    unsafe extern "C" fn unmount_folder(session: u32, mountpoint: *const u8) -> i32 {
        // SAFETY: the wrapper passes a live NUL-terminated string.
        *UNMOUNT_CALL.lock().unwrap() = Some((session, unsafe { read_c_string(mountpoint) }));
        S_OK
    }

    unsafe extern "C" fn create_process(
        session: u32,
        executable: *const u8,
        argv: *mut *const u8,
        environment: *mut *const u8,
        process: *mut WSLCProcessHandle,
        errno: *mut i32,
    ) -> i32 {
        // SAFETY: the wrapper passes initialized pointer arrays and output slots.
        let call = unsafe {
            ProcessCall {
                session_id: session,
                executable: read_c_string(executable),
                argv: read_string_array(argv),
                environment: read_string_array(environment),
            }
        };
        *PROCESS_CALL.lock().unwrap() = Some(call);

        if CREATE_PROCESS_FAILS.load(Ordering::SeqCst) {
            // SAFETY: `errno` is a valid output slot.
            unsafe { *errno = 2 };
            return E_FAIL;
        }

        // SAFETY: `process` is a valid output slot. The non-null sentinel is
        // never dereferenced by the wrapper or callback.
        unsafe { *process = NonNull::<u8>::dangling().as_ptr().cast() };
        S_OK
    }

    unsafe extern "C" fn process_get_fd(
        _process: WSLCProcessHandle,
        fd: i32,
        _handle: *mut HANDLE,
    ) -> i32 {
        FD_CALLS.lock().unwrap().push(fd);
        E_FAIL
    }

    unsafe extern "C" fn process_get_exit_event(
        _process: WSLCProcessHandle,
        _handle: *mut HANDLE,
    ) -> i32 {
        E_FAIL
    }

    unsafe extern "C" fn process_get_exit_code(
        _process: WSLCProcessHandle,
        exit_code: *mut i32,
    ) -> i32 {
        // SAFETY: the wrapper supplies a valid output slot.
        unsafe { *exit_code = 42 };
        S_OK
    }

    unsafe extern "C" fn release_process(_process: WSLCProcessHandle) {
        RELEASE_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    fn api(version: crate::WSLVersion) -> ApiV1 {
        wslpluginapi_sys::WSLPluginAPIV1 {
            Version: version.into(),
            MountFolder: None,
            ExecuteBinary: None,
            PluginError: None,
            ExecuteBinaryInDistribution: None,
            WSLCMountFolder: Some(mount_folder),
            WSLCUnmountFolder: Some(unmount_folder),
            WSLCCreateProcess: Some(create_process),
            WSLCProcessGetFd: Some(process_get_fd),
            WSLCProcessGetExitEvent: Some(process_get_exit_event),
            WSLCProcessGetExitCode: Some(process_get_exit_code),
            WSLCReleaseProcess: Some(release_process),
        }
        .into()
    }

    #[test]
    fn wslc_api_requires_version_2_9_0() {
        let _guard = TEST_LOCK.lock().unwrap();
        let old_api = api(crate::WSLVersion::new(2, 8, 9));
        let current_api = api(crate::WSLVersion::new(2, 9, 0));

        assert!(old_api.wslc().is_err());
        assert!(current_api.wslc().is_ok());
    }

    #[test]
    fn mount_folder_encodes_paths_and_options() {
        let _guard = TEST_LOCK.lock().unwrap();
        let api = api(crate::WSLVersion::new(2, 9, 0));
        let wslc = api.wslc().unwrap();

        wslc.mount_folder(
            WSLCSessionID::new(7),
            r"C:\containers",
            "/mnt/containers",
            true,
        )
        .unwrap();

        assert_eq!(
            *MOUNT_CALL.lock().unwrap(),
            Some(MountCall {
                session_id: 7,
                windows_path: r"C:\containers".to_owned(),
                mountpoint: "/mnt/containers".to_owned(),
                read_only: true,
            })
        );
    }

    #[test]
    fn unmount_folder_encodes_session_and_mountpoint() {
        let _guard = TEST_LOCK.lock().unwrap();
        let api = api(crate::WSLVersion::new(2, 9, 0));
        let wslc = api.wslc().unwrap();

        wslc.unmount_folder(WSLCSessionID::new(9), "/mnt/containers")
            .unwrap();

        assert_eq!(
            *UNMOUNT_CALL.lock().unwrap(),
            Some((9, "/mnt/containers".to_owned()))
        );
    }

    #[test]
    fn command_forwards_argv_environment_and_releases_process() {
        let _guard = TEST_LOCK.lock().unwrap();
        CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
        RELEASE_COUNT.store(0, Ordering::SeqCst);
        let api = api(crate::WSLVersion::new(2, 9, 0));
        let wslc = api.wslc().unwrap();

        let process = wslc
            .new_command(WSLCSessionID::new(11), "/bin/sh")
            .with_arg0("sh")
            .with_args(["-c", "exit 42"])
            .with_environment("HOME=/root")
            .spawn()
            .unwrap();

        assert_eq!(process.exit_code().unwrap(), 42);
        assert_eq!(
            *PROCESS_CALL.lock().unwrap(),
            Some(ProcessCall {
                session_id: 11,
                executable: "/bin/sh".to_owned(),
                argv: vec!["sh".to_owned(), "-c".to_owned(), "exit 42".to_owned()],
                environment: vec!["HOME=/root".to_owned()],
            })
        );
        assert_eq!(RELEASE_COUNT.load(Ordering::SeqCst), 0);

        drop(process);
        assert_eq!(RELEASE_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn process_stream_and_exit_event_failures_are_returned() {
        let _guard = TEST_LOCK.lock().unwrap();
        CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
        FD_CALLS.lock().unwrap().clear();
        let api = api(crate::WSLVersion::new(2, 9, 0));
        let process = api
            .wslc()
            .unwrap()
            .new_command(WSLCSessionID::new(1), "/bin/true")
            .spawn()
            .unwrap();

        assert!(process.stdin().is_err());
        assert!(process.stdout().is_err());
        assert!(process.stderr().is_err());
        assert!(process.exit_event().is_err());
        assert_eq!(
            *FD_CALLS.lock().unwrap(),
            vec![
                WSLCProcessFd::Stdin as i32,
                WSLCProcessFd::Stdout as i32,
                WSLCProcessFd::Stderr as i32,
            ]
        );
    }

    #[test]
    fn command_creation_error_preserves_errno() {
        let _guard = TEST_LOCK.lock().unwrap();
        CREATE_PROCESS_FAILS.store(true, Ordering::SeqCst);
        let api = api(crate::WSLVersion::new(2, 9, 0));
        let wslc = api.wslc().unwrap();

        let error = wslc
            .new_command(WSLCSessionID::new(1), "/missing")
            .spawn()
            .unwrap_err();

        assert_eq!(error.errno(), 2);
        assert_eq!(error.windows_error().code(), HRESULT(E_FAIL));
        CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
    }
}
