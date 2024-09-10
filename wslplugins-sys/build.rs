extern crate bindgen;
extern crate semver;

use bindgen::callbacks::{ParseCallbacks, TypeKind};
use semver::Version;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

const WSL_PACKAGE_NAME: &str = "Microsoft.WSL.PluginApi";
const LOCAL_NUGET_PATH: &str = "nuget_packages"; // Local folder to store NuGet packages

#[derive(Debug)]
struct BindgenCallback {
    generate_hooks_fields_name: bool,
}

impl Default for BindgenCallback {
    fn default() -> Self {
        Self {
            generate_hooks_fields_name: false,
        }
    }
}

impl BindgenCallback {
    fn new(generate_hooks_fields_names: bool) -> Self {
        BindgenCallback {
            generate_hooks_fields_name: generate_hooks_fields_names,
        }
    }
}

impl ParseCallbacks for BindgenCallback {
    fn add_derives(&self, _info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        if _info.kind == TypeKind::Struct && _info.name == "WSLVersion" {
            vec![
                "Eq".into(),
                "PartialEq".into(),
                "Ord".into(),
                "PartialOrd".into(),
                "Hash".into(),
            ]
        } else if _info.kind == TypeKind::Struct
            && _info.name.contains("PluginHooks")
            && self.generate_hooks_fields_name
        {
            vec!["FieldNamesAsSlice".into()]
        } else {
            vec![]
        }
    }
}

// Function to ensure the NuGet package is installed in the local folder
fn ensure_package_installed(
    package_name: &str,
    package_version: &str,
    output_dir: &str,
) -> Result<ExitStatus, Box<dyn std::error::Error>> {
    // Run the NuGet install command with -NonInteractive to avoid prompts
    let status = Command::new("nuget")
        .args(&[
            "install",
            package_name,
            "-Version",
            package_version,
            "-OutputDirectory",
            output_dir,        // Local folder to install the NuGet package
            "-NonInteractive", // Ensures the command runs without user interaction
        ])
        .status()
        .expect("Failed to execute nuget install command");

    if !status.success() {
        return Err(format!(
            "NuGet install command failed with status: {:?}",
            status.code()
        )
        .into());
    }
    Ok(status)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Extract the version of the package from the Cargo metadata
    let version_str = env!("CARGO_PKG_VERSION");
    let version = Version::parse(version_str).expect("Unable to parse the Cargo package version");
    let build_metadata = &version.build;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:version={}", version);
    if !build_metadata.is_empty() {
        println!("cargo:build-metadata={}", build_metadata);
    }

    let package_version = build_metadata.to_string();

    // Ensure the NuGet package is installed in the specified local directory
    ensure_package_installed(WSL_PACKAGE_NAME, &package_version, LOCAL_NUGET_PATH)?;

    // Construct the full path to the installed package in the local directory
    let package_path =
        Path::new(LOCAL_NUGET_PATH).join(format!("{:}.{:}", WSL_PACKAGE_NAME, package_version));

    // Construct the path to the header file
    let header_file_path = package_path
        .join("build")
        .join("native")
        .join("include")
        .join("WslPluginApi.h");

    // Check if the header file exists
    if !header_file_path.exists() {
        return Err(format!("Header file does not exist: {:?}", header_file_path).into());
    }

    println!("Using header file from: {:?}", header_file_path);

    // Use bindgen to generate Rust bindings from the header file
    let hooks_fields_name_feature = env::var("CARGO_FEATURE_HOOKS_FIELD_NAMES").is_ok();
    let mut builder = bindgen::Builder::default()
        .header(header_file_path.to_str().unwrap())
        .raw_line("use windows::core::*;")
        .raw_line("use windows::Win32::Foundation::*;")
        .raw_line("use windows::Win32::Security::*;")
        .raw_line("use windows::Win32::Networking::WinSock::SOCKET;")
        .raw_line("type LPCWSTR = PCWSTR;")
        .raw_line("type LPCSTR = PCSTR;")
        .raw_line("type DWORD = u32;");

    if hooks_fields_name_feature {
        builder = builder.raw_line("use struct_field_names_as_array::FieldNamesAsSlice;");
    }

    let wslplugins_sys = builder
        .derive_debug(true)
        .derive_copy(true)
        .allowlist_item("WSL.*")
        .allowlist_item("Wsl.*")
        .clang_arg("-fparse-all-comments")
        .allowlist_recursively(false)
        .parse_callbacks(Box::new(BindgenCallback::new(hooks_fields_name_feature)))
        .generate_comments(true)
        .generate()
        .expect("Unable to generate wslplugins_sys");

    // Write the generated bindings to the OUT_DIR
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wslplugins_sys_out_path = out_dir.join("wslplugins_sys.rs");
    wslplugins_sys
        .write_to_file(wslplugins_sys_out_path)
        .expect("Couldn't write wslplugins_sys!");

    Ok(())
}
