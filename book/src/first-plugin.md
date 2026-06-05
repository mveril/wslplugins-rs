# Your First Plugin

The minimal shape of a plugin is a Rust `cdylib` crate containing a type plus an implementation of
`WSLPluginV1`.
The `#[wsl_plugin_v1]` macro generates the exported functions and hook wiring expected by WSL.

```rust
# extern crate wslplugins_rs;
use wslplugins_rs::prelude::*;

pub(crate) struct MyPlugin {
    context: &'static WSLContext,
}

#[wsl_plugin_v1]
impl WSLPluginV1 for MyPlugin {
    fn try_new(context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self { context })
    }
}
```

Use a requirement in `#[wsl_plugin_v1(...)]` only when the whole plugin depends on a specific WSL
Plugin API version or named capability. See [Version Capabilities](./version-capabilities.md) for
the supported forms and feature gates.

The `context` gives access to the framework API wrapper. Store it if the plugin needs to call WSL
APIs later from hook methods.

## Add Hooks

Hooks are regular trait methods. Override only the lifecycle events the plugin needs:

```rust
# extern crate wslplugins_rs;
use std::ffi::OsString;
use wslplugins_rs::prelude::*;
use wslplugins_rs::windows_core::HRESULT;

const E_FAIL: HRESULT = HRESULT(0x80004005u32 as i32);

pub(crate) struct MyPlugin {
    context: &'static WSLContext,
}

#[wsl_plugin_v1]
impl WSLPluginV1 for MyPlugin {
    fn try_new(context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self { context })
    }

    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        user_settings: &WSLVmCreationSettings,
    ) -> PluginResult<()> {
        let mut stream = self
            .context
            .api
            .new_command(session, "/bin/cat")
            .with_arg("/proc/version")
            .execute()?;

        let mut kernel_version = String::new();
        std::io::Read::read_to_string(&mut stream, &mut kernel_version).map_err(|error| {
            PluginError::with_message(E_FAIL, OsString::from(error.to_string()))
        })?;
        Ok(())
    }
}
```

Command paths are Linux paths such as `/bin/cat`. `execute()` returns a `TcpStream` connected to the
process stdin and stdout; stderr is forwarded to Linux `dmesg`. This example runs in the VM root
namespace; see [Command Execution](./command-execution.md) for distribution-scoped execution.

## Start from an Example

If you want a complete working reference, start with the `minimal` example from the repository. It
demonstrates:

- creating plugin state in `try_new`;
- logging VM and distribution lifecycle events;
- running `/bin/cat /proc/version` when the VM starts;
- using the macro to declare plugin-wide requirements when a hook or API capability is mandatory.
  New plugins should prefer named capabilities where available; see
  [Version Capabilities](./version-capabilities.md).

In your own plugin crate, the equivalent release build is:

```powershell
cargo build --release
```

After that, sign and register the DLL as described in the packaging chapter.

Next: read [WSL Plugin Model](./wsl-plugin-model.md) for the host model and available events, or
jump to [Packaging and Deployment](./packaging.md) when you are ready to load the DLL into WSL.
