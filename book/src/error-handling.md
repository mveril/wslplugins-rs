# Error Handling

WSL plugins run inside the WSL service process, so failures should be explicit errors instead of
process-level failures. Return errors instead of panicking:

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
