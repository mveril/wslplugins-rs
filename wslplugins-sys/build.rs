extern crate bindgen;
extern crate reqwest;
extern crate semver;
extern crate zip;

use bindgen::callbacks::{ParseCallbacks, TypeKind};
use semver::Version;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use zip::ZipArchive;

#[derive(Debug)]
struct BindgenCallback;

impl ParseCallbacks for BindgenCallback {
    fn add_derives(&self, _info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        if _info.kind == TypeKind::Struct && _info.name == "WSLVersion" {
            ["Eq", "PartialEq", "Ord", "PartialOrd", "Hash"]
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let version_str = env!("CARGO_PKG_VERSION");
    let version = Version::parse(version_str).expect("Unable to parse version");
    let build_metadata = &version.build;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:version={}", version);
    if !build_metadata.is_empty() {
        println!("cargo:build-metadata={}", build_metadata);
    }

    let out_dir = env::var("OUT_DIR")?;
    let header_path = PathBuf::from(&out_dir).join("WslPluginApi.h");

    // Download the nuget package if it not exist.
    if !header_path.exists() {
        let nuget_version = build_metadata.as_str();
        let url = format!("https://api.nuget.org/v3-flatcontainer/microsoft.wsl.pluginapi/{}/microsoft.wsl.pluginapi.{}.nupkg", nuget_version, nuget_version);
        let response = reqwest::blocking::get(&url)?;
        if response.status().is_success() {
            let nupkg_path = PathBuf::from(&out_dir)
                .join(format!("microsoft.wsl.pluginapi.{}.nupkg", nuget_version));
            let mut file = File::create(&nupkg_path)?;
            let content = response.bytes()?;
            file.write_all(&content)?;
            println!("Package downloaded successfully.");

            // Extract header file.
            let mut archive = ZipArchive::new(File::open(&nupkg_path)?)?;
            let mut file = archive.by_name("build/native/include/WslPluginApi.h")?;
            let mut outfile = File::create(&header_path)?;
            io::copy(&mut file, &mut outfile)?;
            println!("Header file extracted.");
        } else {
            println!("Failed to download the package.");
            std::process::exit(1);
        }
    }

    // Use bindgen to generate the binding.
    println!("Generating wslplugins_sys...");
    let wslplugins_sys = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .raw_line("use windows::core::*;")
        .raw_line("use windows::Win32::Foundation::*;")
        .raw_line("use windows::Win32::Security::*;")
        .raw_line("use windows::Win32::Networking::WinSock::SOCKET;")
        .raw_line("type LPCWSTR = PCWSTR;")
        .raw_line("type LPCSTR = PCSTR;")
        .raw_line("type DWORD = u32;")
        .derive_debug(true)
        .derive_copy(true)
        .allowlist_item("WSL.*")
        .allowlist_item("Wsl.*")
        .clang_arg("-fparse-all-comments")
        .allowlist_recursively(false)
        .parse_callbacks(Box::new(BindgenCallback))
        .generate_comments(true)
        .generate()
        .expect("Unable to generate wslplugins_sys");

    let wslplugins_sys_out_path = PathBuf::from(&out_dir).join("wslplugins_sys.rs");
    wslplugins_sys
        .write_to_file(wslplugins_sys_out_path)
        .expect("Couldn't write wslplugins_sys!");

    Ok(())
}
