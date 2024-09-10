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
- PowerShell (for running signing scripts)
- OpenSSL (used in the signing process) [Download OpenSSL](https://slproweb.com/products/Win32OpenSSL.html)
- **SignTool.exe** from the Windows SDK (for signing the plugin) [Download Windows SDK](https://developer.microsoft.com/en-us/windows/downloads/windows-10-sdk/)
- nuget.exe (for downloading [Microsoft's WSL Plugin API](https://www.nuget.org/packages/Microsoft.WSL.PluginApi))

### Important Notes

To sign the plugin correctly, ensure that:

- **`SignTool.exe`** is accessible. It is available through the **Windows SDK**, which you can install separately or access via the **Visual Studio Developer Command Prompt**.
- **OpenSSL** is installed and properly configured in your `$PATH`.
- The script uses **administrator privileges** to register the certificate and trust it on the local machine, so ensure the script is run in **Administrator mode**.

## Usage

- Create a struct that will host the plugin:

```rust
pub(crate) struct Plugin<'a> {
    api: ApiV1<'a>,
}
```

- Implement the plugin infrastructure and add the macro attribute to the implementation:

```rust
#[wsl_plugin_v1(1, 0, 5)]
impl<'a> WSLPluginV1<'a> for Plugin<'a> {
    fn try_new(api: ApiV1<'a>) -> Result<Self> {
        setup_logging()?;
        let plugin = Plugin { api };
        info!("Plugin created");
        Ok(plugin)
    }
    ...
}
```

### Installation and Configuration

#### Building and Signing the Plugin

1. **Ensure Required Tools are Installed**:

   - Install **OpenSSL** and ensure **SignTool.exe** from the **Windows SDK** is in your `$PATH`. If you are unsure about this, you can use the **Visual Studio Developer Command Prompt**, which pre-configures access to `SignTool.exe`.

2. **Build the Plugin**:

   - Navigate to your project directory using `cd path\to\wsl-plugin-rs`.
   - Execute the build command to create the plugin DLL in Debug or Release mode. Use `cargo build --release` for an optimized version.

3. **Sign the Plugin**:

   - After building, sign the plugin to confirm its integrity and origin. Use PowerShell to run the signing script:

   ```powershell
     .\sign-plugin.ps1 -PluginPath .	arget
   elease\plugin.dll -Trust
   ```

- Ensure the path to the DLL is correct and that the `sign-plugin.ps1` script is properly configured to handle Rust DLLs.

**Note**: It is recommended to run this script in the **Visual Studio Developer Command Prompt** to ensure proper access to `SignTool.exe` and administrative rights for signing operations.

#### Registering and Loading the Plugin with WSL

4. **Register the Plugin with WSL**:

   - Use the following command to add the plugin to the Windows registry, allowing WSL to recognize it:

   ```cmd
     reg.exe add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" /v wsl-plugin-rs /d path	o\wsl-plugin-rs	arget
   elease\plugin.dll /t reg_sz
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

   - Once the plugin is loaded, open the file `C:\wsl-plugin.log` to check the plugin output. Ensure that the plugin is correctly writing to this file during its operation.

7. **Troubleshooting**:
   - **Common Issues**:
     - Ensure that the paths to `SignTool.exe` and OpenSSL are correctly set in your environment variables.
     - Verify that the `sign-plugin.ps1` script has been run with elevated permissions (Administrator mode).
     - Check the plugin's build output for any linking or compilation errors.
     - Ensure all the steps for signing and registering the plugin have been followed.
   - If the plugin does not function as expected, check the error logs, ensure all steps of signing and registration were performed correctly, and that file paths are correct. Also, check for any permissions and security settings that might block the plugin's execution.

## To do

- Improve the interface to be more idiomatic rust.
- Bug fixes.
- Publish the crate.

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
