# WSLC API

WSL Plugin API `2.9.0` adds operations for WSL container sessions. These sessions use a distinct
identifier space and process model from regular WSL sessions.

The crate reflects that boundary with two API types:

- `ApiV1` contains the global WSL plugin operations.
- `WSLCApi` contains only operations for WSLC sessions.

Obtain the container API through `ApiV1::wslc()`:

```rust
# extern crate wslplugins_rs;
use wslplugins_rs::prelude::*;

fn container_api(api: &ApiV1) -> ApiResult<WSLCApi<'_>> {
    api.wslc()
}
```

This call checks [`WSLVersionCapability::WSLC`](./version-capabilities.md#wslc). It returns
`ApiError::RequiresUpdate` when the host API is older than `2.9.0`. If the plugin cannot operate
without WSLC, declare the same capability in its macro:

```rust
# extern crate wslplugins_rs;
# use wslplugins_rs::prelude::*;
# struct Plugin;
#[wsl_plugin_v1(WSLVersionCapability::WSLC)]
impl WSLPluginV1 for Plugin {
    fn try_new(_context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self)
    }
}
```

## Mounting a Folder

`WSLCApi::mount_folder` accepts a Windows host path, a UTF-8 Unix mountpoint and a read-only flag.
Use the same mountpoint with `unmount_folder`:

```rust
# extern crate wslplugins_rs;
# use wslplugins_rs::prelude::*;
# fn example(api: &ApiV1, session_id: WSLCSessionID) -> ApiResult<()> {
let wslc = api.wslc()?;
wslc.mount_folder(
    session_id,
    r"C:\container-data",
    "/mnt/container-data",
    true,
)?;
wslc.unmount_folder(session_id, "/mnt/container-data")?;
# Ok(())
# }
```

`WSLCSessionID` is intentionally not interchangeable with `SessionID`.

## Creating a Process

`WSLCApi::new_command` creates a process in the WSLC session's root namespace. Arguments and
environment entries are encoded as NUL-terminated arrays for the native API:

```rust
# extern crate wslplugins_rs;
# use wslplugins_rs::prelude::*;
# fn example(api: &ApiV1, session_id: WSLCSessionID) -> Result<(), Box<dyn std::error::Error>> {
let process = api
    .wslc()?
    .new_command(session_id, "/bin/sh")
    .with_arg0("sh")
    .with_args(["-c", "printf container"])
    .with_environment("HOME=/root")
    .spawn()?;

let stdout = process.stdout()?;
let exit_event = process.exit_event()?;

// Wait for `exit_event` with the Windows synchronization API before reading
// the final exit code.
drop(exit_event);
drop(stdout);
# drop(process);
# Ok(())
# }
```

`spawn` reports both the Windows API failure and the Linux `errno` through
`WSLCCreateProcessError`.

## Resource Lifetime

`WSLCProcess` owns the opaque native process and releases it on drop. Its stdin, stdout, stderr and
exit-event methods return owned Windows handles tied to the process borrow. Rust therefore prevents
the process from being released while one of those handles is still alive, matching the native
WSLC contract.

After the exit event is signaled, call `WSLCProcess::exit_code`. Drop all process handles before
dropping the process.
