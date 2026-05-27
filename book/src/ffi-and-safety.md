# FFI and Safety

WSL plugins run inside the WSL service process. A crash, panic across an FFI boundary, invalid
pointer, or blocking hook can affect the host service. Treat plugin code like systems code even when
using safe Rust wrappers.

## Error Handling

Return errors instead of panicking:

- use `WinResult<T>` for Windows API style errors;
- use `PluginResult<T>` for plugin hook errors;
- map I/O errors into Windows errors where required by the hook signature;
- avoid `panic!`, `unwrap`, and `expect` in plugin code.

Panics are a poor fit for WSL plugins because the plugin is loaded inside a WSL service process, not
inside a short-lived CLI that only affects its own execution. A panic in a hook can unwind through an
FFI boundary, abort the process, poison shared state, or leave WSL with only a generic failure code.
The host is also allowed to call hooks during lifecycle operations such as VM startup, distribution
registration, or shutdown; failing predictably is more useful than terminating the service path.

Avoiding `unwrap` and `expect` is part of the same rule. A missing field, inaccessible file, invalid
distribution name, or failed command execution should become an explicit error that WSL can report
or handle. It should not become an accidental process failure.

## Plugin Errors and Windows Errors

`wslplugins-rs` exposes two result styles because WSL has two different error channels:

- `WinResult<T>` returns a normal Windows `HRESULT` to the host.
- `PluginResult<T>` returns `PluginError`, which contains an `HRESULT` and an optional message.

Use `WinResult<T>` when the hook signature is purely Windows-style or when only an error code is
expected. Use `PluginResult<T>` where the framework hook supports a plugin-level diagnostic message.

When a `PluginError` has a message, the framework sends that message to WSL through the plugin API
before converting the error back into a Windows error code. That gives WSL a human-readable plugin
diagnostic instead of only an `HRESULT`. In practice, this is useful for policy-style failures where
the user should see why an operation was denied:

```rust
# extern crate wslplugins_rs;
use std::ffi::OsString;
use wslplugins_rs::prelude::*;
use wslplugins_rs::windows_core::HRESULT;

const E_ACCESSDENIED: HRESULT = HRESULT(0x80070005u32 as i32);

fn reject_distribution(name: &str) -> PluginResult<()> {
    Err(PluginError::with_message(
        E_ACCESSDENIED,
        OsString::from(format!(
            "Distribution '{name}' is blocked by local policy"
        )),
    ))
}
```

If you convert everything to `windows_core::Error` too early, WSL still receives the failure code,
but it can lose the plugin-specific explanation. Prefer `PluginResult` for hooks where the message
is part of the user experience.

The raw `PluginError` API is intended for synchronous use while WSL is processing VM or distribution
creation. In practice, use `PluginResult` from `on_vm_started` and `on_distribution_started` when a
message should be shown to the user. Do not store a delayed error message and try to report it after
the hook has returned.

## Unsafe Code

Most plugin code should not need raw FFI access. When unsafe code is necessary:

- keep unsafe blocks as small as possible;
- document each block with a `SAFETY:` comment;
- validate pointer lifetimes and ownership before wrapping raw values;
- avoid storing borrowed FFI data beyond the lifetime guaranteed by WSL.

The `wslplugins-rs` crate centralizes raw API handling so plugin authors can usually work with typed
Rust values instead.

## Hook Behavior

Hooks are synchronous notifications. Keep them predictable:

- do the minimum required work in the hook;
- avoid unbounded waits;
- avoid holding locks while calling into WSL APIs;
- make logging best-effort and failure-aware;
- assume hooks may be called multiple times for lifecycle retries.

If a plugin needs heavier work, prefer moving it to a controlled background path with clear
shutdown behavior.

Microsoft's WSL plugin documentation calls out three operational consequences of this model:

- WSL waits for hook callbacks before continuing the lifecycle operation.
- Most plugin errors are fatal to the VM or distribution startup path.
- Plugin code runs in the WSL service process, so a plugin crash can crash the service.

That is why the framework examples keep hooks small and use typed errors instead of process-level
failure mechanisms.

## Command Execution

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
namespace. That path uses `ExecuteBinaryInDistribution`, which requires API version `2.1.2` or newer.

The WSL Plugin API connects the returned socket to stdin and stdout. Stderr is sent to Linux
`dmesg`, so command failures should be designed with that in mind: return explicit hook errors for
user-facing failures, and reserve stderr for diagnostics that can be inspected from the Linux side.

## Advanced FFI Notes

The `sys` feature re-exports the raw `wslpluginapi-sys` bindings for cases where the safe wrapper
does not yet expose a field or function. Prefer the safe wrapper first. If raw access is necessary,
keep these header-level rules in mind:

- hook input pointers are valid only during the callback;
- string pointers from WSL may be null where the header allows it, such as `PackageFamilyName`;
- `ExecuteBinary` argument arrays are null-terminated at the ABI boundary;
- `WSLDistributionInformation::InitPid` was introduced in API version `2.0.5`;
- `Flavor` and `Version` exist in the current header; `wslplugins-rs` exposes them through
  `flavor()` and `version()` and currently requires API version `2.4.4` before reading them.

These details are useful when debugging ABI mismatches, but ordinary plugin logic should stay on the
typed Rust side.
