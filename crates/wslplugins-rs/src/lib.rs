// Enable doc_cfg if docrs
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # WSLPlugin-rs
//!
//! This is the main entry point for the **WSLPlugin-rs** crate, a framework designed for creating
//! plugins for the **Windows Subsystem for Linux (WSL)** using idiomatic Rust.
//!
//! ## Overview
//!
//! WSLPlugin-rs simplifies the development of [WSL plugins](https://learn.microsoft.com/windows/wsl/wsl-plugins?utm_source=chatgpt.com) by providing higher-level abstractions
//! built on top of the raw APIs. This crate exports useful modules and types for plugin creation,
//! such as session management, VM handling, and distribution operations.
//!
//! ## Usage
//!
//! Use the exposed modules and types to build custom WSL plugins. Conditional features like `macro`
//! enable procedural macros for simplifying the plugin development process.
//!
//! ### Example
//!
//! ```rust
//! #[cfg(feature = "macro")]
//! use wslplugins_rs::prelude::*;
//! pub(crate) struct MyPlugin {
//!   context: &'static WSLContext,
//! }
//! #[wsl_plugin_v1(2, 0, 5)]
//! impl WSLPluginV1 for MyPlugin {
//!     fn try_new(context: &'static WSLContext) -> WinResult<Self> {
//!         Ok(MyPlugin { context })
//!     }
//! }
//! ```

/// Provides interfaces for interacting with WSL plugin APIs.
pub mod api;

// Internal modules for managing specific WSL features.
mod core_wsl_distribution_information;
pub(crate) mod cstring_ext;
mod wsl_offline_distribution_information;
mod session_id;
pub mod user_distribution_id;
pub use windows_core;
#[doc(hidden)]
#[cfg(feature = "macro")]
pub mod __private;
pub mod distribution_id;
mod utils;
mod wsl_context;
mod wsl_distribution_information;
mod wsl_session_information;
mod wsl_vm_creation_settings;
#[cfg(doc)]
use crate::plugin::WSLPluginV1;
pub mod wsl_user_configuration;
pub use typed_path;
pub use wsl_user_configuration::WSLUserConfiguration;
/// Tools and utilities for creating custom WSL plugins.
pub mod plugin;
/// Convenient re-exports for common plugin development imports.
pub mod prelude;

// Re-exports for core structures to simplify usage.
pub use core_wsl_distribution_information::CoreWSLDistributionInformation;
pub use distribution_id::DistributionID;
pub use wsl_offline_distribution_information::WSLOfflineDistributionInformation;
pub use wsl_context::WSLContext;
pub use wsl_distribution_information::WSLDistributionInformation;
pub use wsl_session_information::WSLSessionInformation;
pub use wsl_vm_creation_settings::WSLVmCreationSettings;
mod wsl_version;
pub use api::WSLCommandExecution;
#[cfg(feature = "semver")]
pub use wsl_version::SemverConversionError;
pub use session_id::{HasSessionId, SessionID};
pub use user_distribution_id::UserDistributionID;
pub use wsl_version::{WSLVersion, WSLVersionParseError};

/// Re-exports procedural macros when the `macro` feature is enabled.
/// It allow to mark a plugin struct (that implement [`WSLPluginV1`] trait) to be easely integrated to the WSL plugin system without writing manually C code for entry point or hooks.
#[cfg(feature = "macro")]
pub use wslplugins_macro::wsl_plugin_v1;

/// Re-exports the `wslpluginapi_sys` crate as `sys` when the `sys` feature is enabled.
#[cfg(feature = "sys")]
pub use wslpluginapi_sys as sys;
