# Testing

Testing a WSL plugin happens at two levels: ordinary Rust validation for your crate and host
validation on Windows after the DLL is signed and registered.

## Rust Validation

Run normal Rust checks while developing your plugin:

```powershell
cargo test
cargo clippy --all-targets --all-features
cargo fmt -- --check
```

Unit tests are useful for your own parsing, policy, configuration, and decision logic. Keep direct
WSL service behavior behind small boundaries so most of the plugin can be tested without loading a
DLL into the host service.

## Hook-Oriented Tests

Where possible, separate hook code from business logic:

```rust
fn should_block_distribution(name: &str) -> bool {
    name.eq_ignore_ascii_case("blocked")
}
```

Then call that logic from `on_distribution_registered`, `on_vm_started`, or whichever hook applies.
This keeps hook implementations small and makes failures easier to reproduce in ordinary Rust tests.

## Windows Host Validation

Use host validation when you need to prove the DLL works with the real WSL service:

1. Build the target example in release mode.
2. Sign the DLL.
3. Register the DLL under the WSL plugins registry key.
4. Restart the WSL service.
5. Run a WSL command that triggers the plugin.
6. Inspect plugin output and any expected side effects.

For the minimal example, the observable output is written to `C:\wsl-plugin-demo.txt`.

For your own plugin, choose an observable result that fits the behavior: a log file, an allowed or
blocked operation, a command output, or another side effect that confirms the expected hook ran.

Use commands that trigger the hook you are validating:

```powershell
wsl.exe echo "vm startup test"
wsl.exe -d Ubuntu -- echo "distribution startup test"
```

After rebuilding or changing the registry value, restart the WSL service before testing again:

```powershell
Stop-Service -Name "wslservice" -Force
```

If the plugin does not load or the expected hook does not run, use the
[Troubleshooting](./troubleshooting.md) chapter before changing the plugin code.
