# WSLC API

WSL Plugin API `2.9.0` adds operations for WSL container sessions. These sessions use a distinct
identifier space and process model from regular WSL sessions.

> The complete WSLC surface is currently experimental. Enable the `experimental-wslc` crate
> feature; names and data wrappers may evolve while the native API stabilizes.

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
let mut process = api
    .wslc()?
    .new_command(session_id, "/bin/sh")
    .with_arg0("sh")
    .with_args(["-c", "printf container"])
    .with_env("HOME", "/root")
    .pipe_stdout()
    .spawn()?;

let mut stdout = process
    .stdout
    .take()
    .ok_or_else(|| std::io::Error::other("stdout is unavailable"))?;
let mut output = String::new();
std::io::Read::read_to_string(&mut stdout, &mut output)?;
let status = process.wait()?;

drop(stdout);
# drop(process);
# Ok(())
# }
```

`WSLCProcess` follows the shape of `std::process::Child`. Its `stdin`, `stdout`, and `stderr`
fields are optional owned streams, so they can be borrowed in place or moved out with
`Option::take`. `wait`, `try_wait`, and `wait_with_output` provide the corresponding process
lifecycle operations. The command builder also provides `status` and `output` convenience methods:

`spawn` does not acquire standard-stream handles by default. Call `pipe_stdin`, `pipe_stdout`,
`pipe_stderr`, or `pipe_standard_streams` before `spawn` to request only the handles that will be
used. The `output` convenience method requests stdout and stderr automatically.

`wait_timeout` additionally waits for at most a `std::time::Duration` and returns `None` when
the timeout expires. The opaque WSLC process identifier is not a Windows kernel handle, so
`WSLCProcess` does not implement `AsHandle` or `AsRawHandle`. Use `exit_event` when integrating
the process with Windows wait APIs.

Environment variables are supplied as separate key-value pairs with `env`, `with_env`, `envs`, or
`with_envs`. The command encodes each pair as the native `NAME=VALUE` entry expected by WSLC.

Call `prepare` when the same command will be started multiple times. `PreparedWSLCCommand` retains
the encoded executable, arguments, environment, and native pointer arrays, so each call to `spawn`
can invoke WSLC without rebuilding them. Both command types implement `WSLCCommandExecution`.

```rust
# extern crate wslplugins_rs;
# use wslplugins_rs::prelude::*;
# fn example(api: &ApiV1, session_id: WSLCSessionID) -> Result<(), Box<dyn std::error::Error>> {
let output = api
    .wslc()?
    .new_command(session_id, "/bin/echo")
    .with_arg("container")
    .output()?;

assert!(output.status.success());
assert_eq!(output.stdout, b"container\n");
# Ok(())
# }
```

`spawn` reports native process-creation failures, including the Linux `errno`, through
`WSLCSpawnError`. It can also report a failure to acquire one of the process streams or the exit
event.

## Resource Lifetime

`WSLCProcess` and its stream objects share ownership of the opaque native process. Taking a stream
out of the process therefore remains safe: dropping `WSLCProcess` does not call
`WSLCReleaseProcess` until the final detached stream has also been dropped.

`WSLCProcess::wait` and `WSLCProcess::wait_timeout` close an acquired standard input before waiting, matching
`std::process::Child::wait`. The `wait_with_output` and command-level `output` operations read
stdout and stderr concurrently to avoid blocking when either native pipe buffer fills.
`WSLCCommand::status` waits without acquiring stream handles.

The shared process state uses reference counting so detached streams remain safe. Output collection
allocates buffers for the bytes it returns, while `status` avoids those output allocations.

Session hook information exposes its display name as a borrowed `U16CStr` through
`WSLCSessionInformation::display_name`. Use `display_name_os_string` only when an owned Windows
string is required.

## Plugin Hooks

With `experimental-wslc` and `macro` enabled, `WSLPluginV1` exposes six optional WSLC hooks:

- `on_session_created` and `on_session_stopping`;
- `container_started` and `container_stopping`;
- `image_created` and `image_deleted`.

Only methods explicitly implemented by the plugin are installed in the native hook table. The
macro also checks the `WSLC` capability before installing them. Session callbacks receive
`WSLCSessionInformation`; creation callbacks receive typed JSON views
(`WSLCContainerInformation` or `WSLCImageInformation`), while deletion/stopping callbacks receive
a `WSLCObjectId`.
