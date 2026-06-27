# Panic example

This example intentionally panics from a WSL plugin hook.

It demonstrates that the `#[wsl_plugin_v1]` generated FFI boundary catches Rust panics and converts
them to `E_FAIL` instead of letting an unwind cross into WSL. Running this plugin is expected to fail
VM startup with a plugin error that includes the panic message.

The panic is deliberately triggered with an out-of-bounds index so the message is easy to recognize
from WSL:

```text
index out of bounds: the len is 0 but the index is 1
```
