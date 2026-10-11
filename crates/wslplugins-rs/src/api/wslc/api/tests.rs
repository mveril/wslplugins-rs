use super::*;
use crate::api::ApiV1;
use std::ffi::CStr;
use std::fs::OpenOptions;
use std::os::windows::io::IntoRawHandle as _;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use widestring::U16CStr;
use windows_core::HRESULT;
use windows_sys::Win32::System::Threading::CreateEventW;
use wslpluginapi_sys::windows_sys::Win32::Foundation::{E_FAIL, HANDLE, S_OK};
use wslpluginapi_sys::WSLCProcessHandle;

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
static HANDLE_CALLS_FAIL: AtomicBool = AtomicBool::new(false);
static EXIT_EVENT_SIGNALED: AtomicBool = AtomicBool::new(true);
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
    handle: *mut HANDLE,
) -> i32 {
    FD_CALLS.lock().unwrap().push(fd);
    if HANDLE_CALLS_FAIL.load(Ordering::SeqCst) {
        return E_FAIL;
    }
    let mut options = OpenOptions::new();
    if fd == WSLCProcessFd::Stdin as i32 {
        options.write(true);
    } else {
        options.read(true);
    }
    let Ok(file) = options.open("NUL") else {
        return E_FAIL;
    };
    // SAFETY: ownership of this live handle is transferred to the wrapper.
    unsafe { *handle = file.into_raw_handle() };
    S_OK
}

unsafe extern "C" fn process_get_exit_event(
    _process: WSLCProcessHandle,
    handle: *mut HANDLE,
) -> i32 {
    if HANDLE_CALLS_FAIL.load(Ordering::SeqCst) {
        return E_FAIL;
    }
    let initial_state = i32::from(EXIT_EVENT_SIGNALED.load(Ordering::SeqCst));
    // SAFETY: see `process_get_fd`.
    unsafe { *handle = CreateEventW(ptr::null(), 1, initial_state, ptr::null()) };
    S_OK
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
    HANDLE_CALLS_FAIL.store(false, Ordering::SeqCst);
    RELEASE_COUNT.store(0, Ordering::SeqCst);
    let api = api(crate::WSLVersion::new(2, 9, 0));
    let wslc = api.wslc().unwrap();

    let mut process = wslc
        .new_command(WSLCSessionID::new(11), "/bin/sh")
        .with_arg0("sh")
        .with_args(["-c", "exit 42"])
        .with_env("HOME", "/root")
        .spawn()
        .unwrap();

    assert_eq!(
        process
            .wait_timeout(std::time::Duration::from_secs(1))
            .unwrap()
            .unwrap()
            .code(),
        42
    );
    assert_eq!(process.try_wait().unwrap().unwrap().code(), 42);
    assert_eq!(process.wait().unwrap().code(), 42);
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
fn command_borrows_static_and_local_strings_without_copying() {
    let _guard = TEST_LOCK.lock().unwrap();
    let api = api(crate::WSLVersion::new(2, 9, 0));
    let wslc = api.wslc().unwrap();
    let program = "/bin/echo";
    let static_argument = "static";
    let local_argument = String::from("local");
    let environment_key = "HOME";
    let environment_value = String::from("/root");
    let command = wslc
        .new_command(WSLCSessionID::new(11), program)
        .with_args([static_argument, local_argument.as_str()])
        .with_env(environment_key, environment_value.as_str());

    assert!(ptr::eq(
        command.get_program().as_str().as_ptr(),
        program.as_ptr()
    ));
    let mut arguments = command.get_args();
    assert!(arguments
        .next()
        .is_some_and(|argument| ptr::eq(argument.as_ptr(), static_argument.as_ptr())));
    assert!(arguments
        .next()
        .is_some_and(|argument| ptr::eq(argument.as_ptr(), local_argument.as_ptr())));
    assert!(command
        .get_environment()
        .next()
        .is_some_and(|(key, value)| {
            ptr::eq(key.as_ptr(), environment_key.as_ptr())
                && ptr::eq(value.as_ptr(), environment_value.as_ptr())
        }));
}

#[test]
fn prepared_command_can_spawn_multiple_processes_without_reencoding() {
    let _guard = TEST_LOCK.lock().unwrap();
    CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
    HANDLE_CALLS_FAIL.store(false, Ordering::SeqCst);
    RELEASE_COUNT.store(0, Ordering::SeqCst);
    let api = api(crate::WSLVersion::new(2, 9, 0));
    let command = api
        .wslc()
        .unwrap()
        .new_command(WSLCSessionID::new(12), "/bin/echo")
        .with_arg("prepared")
        .with_env("LANG", "C");
    let prepared = command.prepare();

    drop(prepared.spawn().unwrap());
    drop(prepared.spawn().unwrap());

    assert_eq!(RELEASE_COUNT.load(Ordering::SeqCst), 2);
    assert_eq!(
        *PROCESS_CALL.lock().unwrap(),
        Some(ProcessCall {
            session_id: 12,
            executable: "/bin/echo".to_owned(),
            argv: vec!["/bin/echo".to_owned(), "prepared".to_owned()],
            environment: vec!["LANG=C".to_owned()],
        })
    );
}

#[test]
fn detached_stream_keeps_native_process_alive() {
    let _guard = TEST_LOCK.lock().unwrap();
    CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
    HANDLE_CALLS_FAIL.store(false, Ordering::SeqCst);
    RELEASE_COUNT.store(0, Ordering::SeqCst);
    let api = api(crate::WSLVersion::new(2, 9, 0));
    let mut process = api
        .wslc()
        .unwrap()
        .new_command(WSLCSessionID::new(11), "/bin/true")
        .pipe_stdout()
        .spawn()
        .unwrap();
    let stdout = process.stdout.take().unwrap();

    drop(process);
    assert_eq!(RELEASE_COUNT.load(Ordering::SeqCst), 0);

    drop(stdout);
    assert_eq!(RELEASE_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn spawn_only_acquires_explicitly_requested_streams() {
    let _guard = TEST_LOCK.lock().unwrap();
    CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
    HANDLE_CALLS_FAIL.store(false, Ordering::SeqCst);
    FD_CALLS.lock().unwrap().clear();
    let api = api(crate::WSLVersion::new(2, 9, 0));
    let wslc = api.wslc().unwrap();

    let process = wslc
        .new_command(WSLCSessionID::new(11), "/bin/true")
        .spawn()
        .unwrap();

    assert!(process.stdin.is_none());
    assert!(process.stdout.is_none());
    assert!(process.stderr.is_none());
    assert!(FD_CALLS.lock().unwrap().is_empty());
}

#[test]
fn output_only_acquires_stdout_and_stderr() {
    let _guard = TEST_LOCK.lock().unwrap();
    CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
    HANDLE_CALLS_FAIL.store(false, Ordering::SeqCst);
    EXIT_EVENT_SIGNALED.store(true, Ordering::SeqCst);
    FD_CALLS.lock().unwrap().clear();
    let api = api(crate::WSLVersion::new(2, 9, 0));

    api.wslc()
        .unwrap()
        .new_command(WSLCSessionID::new(11), "/bin/true")
        .output()
        .unwrap();

    assert_eq!(
        *FD_CALLS.lock().unwrap(),
        vec![WSLCProcessFd::Stdout as i32, WSLCProcessFd::Stderr as i32]
    );
}

#[test]
fn command_status_discards_output_and_returns_exit_status() {
    let _guard = TEST_LOCK.lock().unwrap();
    CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
    HANDLE_CALLS_FAIL.store(false, Ordering::SeqCst);
    EXIT_EVENT_SIGNALED.store(true, Ordering::SeqCst);
    let api = api(crate::WSLVersion::new(2, 9, 0));

    let status = api
        .wslc()
        .unwrap()
        .new_command(WSLCSessionID::new(11), "/bin/true")
        .status()
        .unwrap();

    assert_eq!(status.code(), 42);
}

#[test]
fn wait_timeout_returns_none_while_process_is_running() {
    let _guard = TEST_LOCK.lock().unwrap();
    CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
    HANDLE_CALLS_FAIL.store(false, Ordering::SeqCst);
    EXIT_EVENT_SIGNALED.store(false, Ordering::SeqCst);
    let api = api(crate::WSLVersion::new(2, 9, 0));
    let wslc = api.wslc().unwrap();
    let mut process = wslc
        .new_command(WSLCSessionID::new(11), "/bin/sleep")
        .spawn()
        .unwrap();

    assert_eq!(
        process.wait_timeout(std::time::Duration::ZERO).unwrap(),
        None
    );
    assert!(process.stdin.is_none());
    assert_eq!(process.try_wait().unwrap(), None);

    EXIT_EVENT_SIGNALED.store(true, Ordering::SeqCst);
}

#[test]
fn process_stream_and_exit_event_failures_are_returned() {
    let _guard = TEST_LOCK.lock().unwrap();
    CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
    HANDLE_CALLS_FAIL.store(true, Ordering::SeqCst);
    FD_CALLS.lock().unwrap().clear();
    let api = api(crate::WSLVersion::new(2, 9, 0));
    let error = api
        .wslc()
        .unwrap()
        .new_command(WSLCSessionID::new(1), "/bin/true")
        .pipe_stdin()
        .spawn()
        .unwrap_err();

    assert!(matches!(error, WSLCSpawnError::Api(_)));
    assert_eq!(*FD_CALLS.lock().unwrap(), vec![WSLCProcessFd::Stdin as i32]);
}

#[test]
fn command_creation_error_preserves_errno() -> std::result::Result<(), &'static str> {
    let _guard = TEST_LOCK.lock().unwrap();
    CREATE_PROCESS_FAILS.store(true, Ordering::SeqCst);
    HANDLE_CALLS_FAIL.store(false, Ordering::SeqCst);
    let api = api(crate::WSLVersion::new(2, 9, 0));
    let wslc = api.wslc().unwrap();

    let error = wslc
        .new_command(WSLCSessionID::new(1), "/missing")
        .spawn()
        .unwrap_err();

    let WSLCSpawnError::Create(error) = error else {
        return Err("expected a create-process error");
    };
    assert_eq!(error.errno(), 2);
    assert_eq!(error.windows_error().code(), HRESULT(E_FAIL));
    CREATE_PROCESS_FAILS.store(false, Ordering::SeqCst);
    Ok(())
}
