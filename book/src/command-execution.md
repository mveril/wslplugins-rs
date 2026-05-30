# Command Execution

`ApiV1::new_command` builds a Linux command to run in a WSL session. The program path and arguments
must be valid for the target Linux environment:

```rust
# extern crate wslplugins_rs;
use std::ffi::OsString;
use wslplugins_rs::prelude::*;
use wslplugins_rs::windows_core::HRESULT;

const E_FAIL: HRESULT = HRESULT(0x80004005u32 as i32);

fn read_kernel_version(
    context: &WSLContext,
    session: &WSLSessionInformation,
) -> PluginResult<String> {
    let mut stream = context
        .api
        .new_command(session, "/bin/cat")
        .with_arg("/proc/version")
        .execute()?;

    let mut kernel_version = String::new();
    std::io::Read::read_to_string(&mut stream, &mut kernel_version).map_err(|error| {
        PluginError::with_message(E_FAIL, OsString::from(error.to_string()))
    })?;

    Ok(kernel_version)
}
```

The returned stream is connected to stdin and stdout. Drop it when no more interaction is needed.

By default, `WSLCommand` targets the system/root namespace through the WSL API `ExecuteBinary`. This
namespace is not the same as a user distribution. Microsoft documents it as a minimal Mariner-based
root filesystem backed by writable `tmpfs`, so changes made there disappear when the WSL2 VM shuts
down.

Use a user-distribution target when the command must run inside a distribution rather than the root
namespace. That path uses `ExecuteBinaryInDistribution`, which is represented by
`WSLVersionCapability::ExecuteBinaryInDistribution`; see
[Version Capabilities](./version-capabilities.md) for the required API support. If that command path
is central to the plugin, declare the capability in `#[wsl_plugin_v1(...)]`. If it is optional,
handle the error returned by `execute()`.

The returned `TcpStream` is connected to the command's stdin and stdout. It does not carry stderr;
WSL sends stderr output to Linux `dmesg`.
