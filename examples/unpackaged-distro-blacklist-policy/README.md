# Unpackaged distro blacklist policy example

This example demonstrates a WSL plugin that blocks unpackaged distributions at startup.

It is intended as a reference for:
- enforcing a simple host-side policy with [`wslplugins_rs`]
- inspecting [`WSLDistributionInformation`] when a distribution starts
- returning a structured [`PluginError`] to deny an operation
- emitting file-based diagnostics with [`tracing`]

## What this example does

When a distribution starts, the plugin checks its package family name:
- if the distribution has a non-empty package family name, startup is allowed
- if the package family name is missing or empty, startup is denied with `E_ACCESSDENIED`

This makes the example useful for environments that only want to allow Store-packaged or otherwise packaged WSL distributions.

## Error reporting in WSL

The denial path uses [`PluginError::with_message`] to attach a human-readable message to the `E_ACCESSDENIED` error returned by the plugin.

In practice, this is the part of the example that shows how to surface a policy error back to WSL so the user sees a clear explanation instead of only a raw failure code.

## Logging behavior

The plugin writes diagnostics to a file using [`tracing_subscriber`] and [`tracing_appender`].

It supports the following environment variables:
- `RUST_WSL_LOGLEVEL` to configure the log filter
- `RUST_LOG` as a fallback log filter
- `RUST_WSL_LOG_PATH` to override the default log file path

If `RUST_WSL_LOG_PATH` is not set, logs are appended to `C:\wsl-unpackaged-distro-blacklist.log`.
