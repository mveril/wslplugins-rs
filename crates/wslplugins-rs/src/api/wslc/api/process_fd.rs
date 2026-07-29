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
