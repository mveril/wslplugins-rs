# Introduction

`wslplugins-rs` is a Rust framework for building DLL plugins loaded by Windows Subsystem for Linux.
It wraps the raw WSL plugin API with Rust types and provides a procedural macro that generates the
exported entry points and hook registration code expected by WSL.

Use it when your plugin needs to observe WSL lifecycle events, inspect distribution metadata, or run
commands inside a WSL session from plugin code.

This book is written for plugin authors using the library in their own crate. It focuses on the path
from an empty Rust plugin project to a signed and registered DLL on Windows. For item-by-item API
reference, use the generated Rust documentation on docs.rs.

## Design Goals

The framework keeps the plugin authoring model small:

- implement `WSLPluginV1` for a Rust type that owns the plugin state;
- annotate the implementation with `#[wsl_plugin_v1(...)]`;
- use typed wrappers for WSL sessions, distributions, versions, and command execution;
- return errors instead of panicking inside the WSL service process.

WSL loads plugins into a host service process. Reliability and conservative error handling matter
more than convenience shortcuts.

## What You Build

A plugin built with `wslplugins-rs` is a Rust library crate compiled as a Windows DLL:

```toml
[lib]
crate-type = ["cdylib"]
```

The DLL is then signed, registered under the WSL plugins registry key, and loaded by the WSL service.
The examples in the repository are useful starting points, but the same model applies to any plugin
crate that depends on `wslplugins-rs`.
