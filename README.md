# WSLPlugins-rs

WSLPlugins-rs is a project aiming to provide a Rust interface for creating WSL (Windows Subsystem for Linux) plugins. This approach is inspired by [Microsoft's C sample for WSL plugins](https://github.com/microsoft/wsl-plugin-sample), aiming to leverage Rust's safety and performance features and allowing easy building of WSL plugins using idiomatic Rust.

Please note: As this project is in an early stage of development, the API surface is subject to change at any time.

The current state is experimental, and macros will be developed to simplify the plugin development experience by avoiding manual writing of any unsafe code.

## Features

- **Rust Interface**: Leverage the safety and idiomatic Rust features of Rust for developing robust WSL plugins.
- **Extensible Plugin System**: Easily extendable framework for adding new functionalities to WSL.

## Prerequisites

Ensure you have the following requirements installed:

- Rust (latest stable version)
- Cargo (Rust's package manager)
- PowerShell (for running signin script)

## Usage

### Installation and Configuration

#### Building and Signing the Plugin

1. **Open a Visual Studio Developer Command Prompt as Administrator**:

   - This is necessary to ensure proper privileges when building and signing the plugin.

2. **Build the Plugin**:

   - Navigate to your project directory using `cd path\to\wsl-plugin-rs`.
   - Execute the build command to create the plugin DLL in Debug or Release mode. Use `cargo build --release` for an optimized version.

3. **Sign the Plugin**:
   - After building, sign the plugin to confirm its integrity and origin. Use PowerShell to run the signing script:
     ```powershell
     .\sign-plugin.ps1 -PluginPath .\target\release\plugin.dll -Trust
     ```
   - Ensure the path to the DLL is correct and that the `sign-plugin.ps1` script is properly configured to handle Rust DLLs.

#### Registering and Loading the Plugin with WSL

4. **Register the Plugin with WSL**:

   - Use the following command to add the plugin to the Windows registry, allowing WSL to recognize it:
     ```cmd
     reg.exe add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" /v wsl-plugin-rs /d path\to\wsl-plugin-rs\target\release\plugin.dll /t reg_sz
     ```
   - Replace `path\to\wsl-plugin-rs\target\release\plugin.dll` with the exact path of the signed DLL.

5. **Restart WSL Service**:
   - For the plugin to be loaded by WSL, you need to restart the associated service:
     ```cmd
     sc.exe stop wslservice
     sc.exe start wslservice
     ```

#### Verification and Troubleshooting

6. **Verify Plugin Functionality**:

   - Once the plugin is loaded, open the file `C:\wsl-plugin-demo.txt` to check the plugin output. Ensure that the plugin is correctly writing to this file during its operation.

7. **Troubleshooting**:
   - If the plugin does not function as expected, check the error logs, ensure all steps of signing and registration were performed correctly, and that file paths are correct. Also look for any permissions and security settings that might block the plugin's execution.

This section provides developers with all the necessary information to build, deploy, and test their WSL plugins using Rust, ensuring they follow best security practices and system maintenance.

## To do

- Bug fixes.
- Add proc macro in order to generate what we have in the lib.rs file using a code like

```rust
#[wsl_plugin]
impl WSLPluginV1 for Plugin {
 ...
}
```

## Contributing

Contributions to WSLPlugins-rs are welcome! If you have improvements or bug fixes:

1. Fork the repository.
2. Create a new branch for your changes.
3. Develop and test your changes.
4. Submit a pull request with a comprehensive description of changes.

## License

WSLPlugins-rs is released under the MIT License. For more information, please check the LICENSE file in the repository.

## Contact

For support or to contact the developers, please open an issue on the GitHub project page.

### Additional Information

- **Development Notes**: More information on plugin development and system architecture will be added as the project evolves.
