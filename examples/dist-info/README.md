# Dist-info example

This example demonstrates a WSL plugin centered on distribution metadata instead of basic lifecycle logging.

It is intended as a reference for:
- wiring a plugin with [`wslplugins_rs`]
- working with distribution-related types exposed by the framework
- enriching host-side information with Linux `os-release` data through [`rs_release`]
- emitting structured diagnostics with [`tracing`]

Compared with the minimal example, this sample focuses on distro identity and descriptive metadata rather than on reproducing Microsoft's sample plugin behavior.

## What this example includes

The example package enables the following `wslplugins-rs` features:
- `macro` to generate the WSL plugin entry points and hook registration
- `bitflags` for the flag-based framework types used by WSL metadata APIs
- `tracing` so the plugin can report structured runtime information

Additional dependencies are used for documentation-friendly observability:
- [`tracing_subscriber`] and [`tracing_appender`] to configure log output
- [`rs_release`] to parse Linux distribution metadata when it is available
