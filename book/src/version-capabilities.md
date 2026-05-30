# Version Capabilities

WSL Plugin API features are introduced over time. `wslplugins-rs` models those feature gates with
`WSLVersionCapability`, so plugin code can name the API feature it depends on instead of hard-coding
the version number everywhere.

Use capabilities when the crate exposes one for the feature you need. Use an explicit version only
when the plugin depends on an API surface that does not yet have a named capability. The capability
matrix below is the canonical list for this book; other chapters link here instead of repeating it.

## Capability Reference

| Capability | Enables | Minimum API version |
| --- | --- | --- |
| `DistributionInitPid` | `WSLDistributionInformation::init_pid()` | `2.0.5` |
| `DistributionRegisteredHook` | `on_distribution_registered` hook wiring | `2.1.2` |
| `DistributionUnregisteredHook` | `on_distribution_unregistered` hook wiring | `2.1.2` |
| `ExecuteBinaryInDistribution` | command execution inside a user distribution | `2.1.2` |
| `DistributionFlavor` | `CoreWSLDistributionInformation::flavor()` | `2.4.4` |
| `DistributionVersion` | `CoreWSLDistributionInformation::version()` | `2.4.4` |

For command execution details, see [Command Execution](./command-execution.md). For the lifecycle
and registration events exposed by `WSLPluginV1`, see [WSL Plugin Model](./wsl-plugin-model.md).

## Plugin Requirements

The `#[wsl_plugin_v1]` macro checks the plugin-wide API requirement before `try_new` runs:

```rust
# extern crate wslplugins_rs;
use wslplugins_rs::prelude::*;

pub(crate) struct Plugin {
    context: &'static WSLContext,
}

#[wsl_plugin_v1]
impl WSLPluginV1 for Plugin {
    fn try_new(context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self { context })
    }
}
```

No argument means the plugin only requires the base entry point. If the whole plugin needs a known
minimum API version, pass that version explicitly:

```rust
# extern crate wslplugins_rs;
# use wslplugins_rs::prelude::*;
# pub(crate) struct Plugin;
#[wsl_plugin_v1(2, 1)]
impl WSLPluginV1 for Plugin {
    fn try_new(_context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self)
    }
}
```

The revision component is optional and defaults to `0`:

```rust
# extern crate wslplugins_rs;
# use wslplugins_rs::prelude::*;
# pub(crate) struct Plugin;
#[wsl_plugin_v1(2, 1, 2)]
impl WSLPluginV1 for Plugin {
    fn try_new(_context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self)
    }
}
```

When a named capability exists, prefer it:

```rust
# extern crate wslplugins_rs;
# use wslplugins_rs::prelude::*;
# pub(crate) struct Plugin;
#[wsl_plugin_v1(WSLVersionCapability::ExecuteBinaryInDistribution)]
impl WSLPluginV1 for Plugin {
    fn try_new(_context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self)
    }
}
```

Capabilities can be combined with `|`. The macro uses the highest minimum API version required by
the listed capabilities:

```rust
# extern crate wslplugins_rs;
# use wslplugins_rs::prelude::*;
# pub(crate) struct Plugin;
#[wsl_plugin_v1(
    WSLVersionCapability::DistributionRegisteredHook
        | WSLVersionCapability::DistributionUnregisteredHook
)]
impl WSLPluginV1 for Plugin {
    fn try_new(_context: &'static WSLContext) -> WinResult<Self> {
        Ok(Self)
    }
}
```

If the host WSL Plugin API is too old for the plugin-wide requirement, initialization fails with
`WSL_E_PLUGIN_REQUIRES_UPDATE`. WSL does not call `try_new`, and the hook table is not registered
for that plugin.

## Runtime Checks

Plugin-wide requirements are the right choice when the plugin cannot behave correctly without the
feature. Optional features should stay as runtime `Result` checks:

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

The same rule applies to command execution. If distribution-scoped command execution is central to
the plugin, declare `WSLVersionCapability::ExecuteBinaryInDistribution` in `#[wsl_plugin_v1(...)]`.
If it is optional, handle the API error returned by `execute()`.

## Inspecting Versions

`WSLVersionCapability::required_version()` returns the minimum API version for one capability.
`WSLVersion::supports(capability)` checks whether a runtime API version supports a capability, and
`WSLVersion::capabilities()` iterates over the capabilities supported by that version.

```rust
# extern crate wslplugins_rs;
use wslplugins_rs::{WSLVersion, WSLVersionCapability};

let version = WSLVersion::new(2, 1, 2);

assert!(version.supports(WSLVersionCapability::ExecuteBinaryInDistribution));
```
