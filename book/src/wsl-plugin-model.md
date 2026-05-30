# WSL Plugin Model

WSL plugins are native DLLs loaded by the WSL service. The WSL plugin API calls exported functions
from the DLL and sends synchronous notifications for lifecycle events.

`wslplugins-rs` turns that model into a Rust trait:

- `try_new` creates the plugin state.
- `on_vm_started` runs after a VM starts.
- `on_vm_stopping` runs before a VM stops.
- `on_distribution_started` runs after a distribution starts.
- `on_distribution_stopping` runs before a distribution stops.
- `on_distribution_registered` runs when a distribution is registered.
- `on_distribution_unregistered` runs when a distribution is unregistered.

The default hook implementations do nothing and return success, so a plugin only implements the
events it needs.

## What WSL Loads

At the ABI level, a plugin exposes the `WSLPluginAPIV1_EntryPoint` entry point. WSL passes that
entry point a `WSLPluginAPIV1` table and expects the plugin to fill a `WSLPluginHooksV1` table with
the callbacks it wants to handle.

The `#[wsl_plugin_v1(...)]` macro generates this entry point and hook table wiring for an
implementation of `WSLPluginV1`. Most plugin crates should use the macro instead of writing the
entry point manually.

The API table contains:

- the WSL plugin API version currently offered by the host;
- `MountFolder`, for creating a Plan 9 mount between Windows and Linux;
- `ExecuteBinary`, for running a program in the root namespace of the WSL2 VM;
- `PluginError`, for passing a user-facing failure message back to WSL;
- `ExecuteBinaryInDistribution`, introduced in API version `2.1.2`, for running a program inside a
  user distribution.

The hook table contains VM lifecycle hooks, distribution lifecycle hooks, and distribution
registration hooks. The registration hooks were introduced in API version `2.1.2`.

## Plugin State

The plugin type owns state that must live for the loaded plugin lifetime. The WSL context gives
access to the API wrapper:

```rust
# extern crate wslplugins_rs;
# use wslplugins_rs::prelude::*;
pub(crate) struct Plugin {
    context: &'static WSLContext,
}
```

The trait requires `Sync` because the host may call plugin hooks from service-owned execution paths.
Shared state should use synchronization primitives that are appropriate for service code and should
avoid long blocking critical sections.

The raw WSL structs passed to hooks are borrowed from WSL. The Microsoft header documents those
pointers as valid only while the hook call is in progress. `wslplugins-rs` exposes typed wrapper
references for those values, but the same lifetime rule still matters: copy out the data you need if
it must outlive the hook.

## API Wrappers

The public crate provides typed wrappers for common WSL concepts:

- `WSLContext`: plugin context and access to `ApiV1`.
- `WSLSessionInformation`: current WSL session metadata.
- `WSLDistributionInformation`: online distribution metadata.
- `WSLOfflineDistributionInformation`: offline distribution metadata used by registration hooks.
- `WSLVmCreationSettings`: VM creation settings.
- `SessionID`, `DistributionID`, and `UserDistributionID`: typed identifiers.
- `WSLVersion`: parsed WSL version values.

Prefer these wrappers over raw FFI types in plugin code. Raw access is available behind the `sys`
feature for cases where a wrapper does not yet expose the needed API.

Online and offline distribution wrappers both implement `CoreWSLDistributionInformation`. That gives
common access to the distribution ID, name, optional package family name, flavor, and version. The
crate checks the runtime API version for fields that were added later: `init_pid()` requires
`2.0.5`, and `flavor()` / `version()` require `2.4.4`.

## Versioned Hooks

Some hooks are only available in newer WSL plugin API versions. For example, distribution
registration hooks are documented in this crate as introduced in API version `2.1.2`.

Choose the version passed to `#[wsl_plugin_v1(...)]` according to the hooks and API calls your
plugin uses. A plugin that only handles basic VM lifecycle events can target an older version than a
plugin that observes distribution registration.

The macro accepts `major, minor` or `major, minor, revision`. If the revision is omitted, it is
treated as `0`.

If WSL offers an older API version than the one requested by the plugin macro, initialization fails
with `WSL_E_PLUGIN_REQUIRES_UPDATE`. This is preferable to registering callbacks that rely on fields
or function pointers the host does not provide.

## Version Checks

`wslplugins-rs` uses two complementary version checks.

The first check happens when WSL loads the DLL. The version passed to
`#[wsl_plugin_v1(...)]` is the minimum API version required by the plugin as a whole. During entry
point initialization, the generated code compares that requirement with `context.api.version()`. If
the runtime API is too old, plugin creation stops and WSL receives
`WSL_E_PLUGIN_REQUIRES_UPDATE`.

That means WSL does not call `try_new` and the hook table is not registered for that plugin. A hook
that requires a newer API version will therefore never be called if you declare that version in the
macro and the host runtime is older. This is the right choice for features that are central to the
plugin's behavior.

Hook availability is decided when the plugin is registered with WSL, not lazily when an event
happens. If the current WSL plugin API version does not provide a hook slot, the method associated
with that event is never called. For example, on a runtime older than `2.1.2`, distribution
registration and unregistration notifications are not available, so
`on_distribution_registered` and `on_distribution_unregistered` cannot run.

The second check happens at runtime on individual APIs or fields. Some wrappers return a `Result`
instead of silently reading a field that may not exist on the current WSL API version:

```rust
# extern crate wslplugins_rs;
use wslplugins_rs::prelude::*;

fn describe_distribution(distribution: &WSLDistributionInformation) -> PluginResult<String> {
    let init_pid = distribution.init_pid()?;
    Ok(format!(
        "{} is running with init PID {init_pid}",
        distribution.name().to_string_lossy()
    ))
}
```

If the runtime API is too old for `init_pid()`, the call returns a `RequiresUpdate` error that maps
to `WSL_E_PLUGIN_REQUIRES_UPDATE`. The same pattern is used by distribution-scoped command
execution: `ExecuteBinaryInDistribution` requires API version `2.1.2`, so
`with_distribution_id(DistributionID::User(...)).execute()` can return an API error on an older
runtime.

Use runtime `Result` checks when the feature is optional:

```rust
# extern crate wslplugins_rs;
use wslplugins_rs::prelude::*;

fn optional_flavor(distribution: &WSLDistributionInformation) -> Option<String> {
    distribution
        .flavor()
        .ok()
        .flatten()
        .map(|value| value.to_string_lossy().into_owned())
}
```

Use the macro version requirement when the plugin cannot behave correctly without the newer hook,
field, or API call. For example, a plugin whose main purpose is to run commands inside a specific
user distribution should require at least `2.1.2`; a plugin that only uses `flavor()` as a nicer log
detail can keep a lower macro version and treat that field as optional.

## Hook Failure Semantics

Most hook failures are treated as lifecycle failures by WSL. For example, an error from
`OnVMStarted` or `OnDistributionStarted` can prevent the VM or distribution from starting.

Distribution registration notifications are different in the current Microsoft header: failures from
`OnDistributionRegistered` and `OnDistributionUnregistered` do not cause the register or unregister
operation itself to fail. Use those hooks for observation, cleanup, indexing, or best-effort policy
work. If you need to block startup, enforce that policy in a startup hook instead.

## External References

- Microsoft WSL plugin documentation:
  <https://learn.microsoft.com/en-us/windows/wsl/wsl-plugins>
- Microsoft WSL Plugin API NuGet package:
  <https://www.nuget.org/packages/Microsoft.WSL.PluginApi>
- Microsoft WSL plugin sample:
  <https://github.com/microsoft/wsl-plugin-sample>
