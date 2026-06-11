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
//! For a more complete user guide, see the [wslplugins-rs Book](https://mveril.github.io/wslplugins-rs/).
//!
//! ## Usage
//!
//! Use the exposed modules and types to build custom WSL plugins. The `macro`
//! feature enables the [`wsl_plugin_v1`] attribute, which generates the WSL
//! Plugin API entry point and hook wiring for a [`WSLPluginV1`] implementation.
//!
//! The macro can be used without arguments for the base API, with an explicit
//! minimum version, or with one or more [`WSLVersionCapability`] values when the
//! plugin depends on named API capabilities.
//!
//! ### Example
//!
//! ```rust
//! # #[cfg(feature = "macro")]
//! # mod example {
//! use wslplugins_rs::prelude::*;
//! pub(crate) struct MyPlugin {
//!   context: &'static WSLContext,
//! }
//! #[wsl_plugin_v1]
//! impl WSLPluginV1 for MyPlugin {
//!     fn try_new(context: &'static WSLContext) -> WinResult<Self> {
//!         Ok(MyPlugin { context })
//!     }
//! }
//! # }
//! ```

/// Provides interfaces for interacting with WSL plugin APIs.
pub mod api;

// Internal modules for managing specific WSL features.
mod core_wsl_distribution_information;
pub(crate) mod cstring_ext;
mod session_id;
pub mod user_distribution_id;
mod wsl_offline_distribution_information;
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
/// Re-exports the `win-security-identifier` crate for working with Windows SIDs.
pub use win_security_identifier;
pub use wsl_user_configuration::WSLUserConfiguration;
/// Tools and utilities for creating custom WSL plugins.
pub mod plugin;
/// Convenient re-exports for common plugin development imports.
pub mod prelude;

// Re-exports for core structures to simplify usage.
pub use core_wsl_distribution_information::CoreWSLDistributionInformation;
pub use distribution_id::DistributionID;
pub use wsl_context::WSLContext;
pub use wsl_distribution_information::WSLDistributionInformation;
pub use wsl_offline_distribution_information::WSLOfflineDistributionInformation;
pub use wsl_session_information::WSLSessionInformation;
pub use wsl_vm_creation_settings::WSLVmCreationSettings;
mod wsl_version;
pub use api::WSLCommandExecution;
#[cfg(feature = "semver")]
pub use wsl_version::SemverConversionError;
pub use wsl_version::WSLVersionCapability;
pub use wsl_version::{WSLVersion, WSLVersionParseError};

/// Re-exports procedural macros when the `macro` feature is enabled.
///
/// Use [`wsl_plugin_v1`] on a [`WSLPluginV1`] implementation to generate the
/// exported WSL Plugin API entry point and hook table setup without writing the
/// C ABI glue manually.
#[cfg(feature = "macro")]
pub use wslplugins_macro::wsl_plugin_v1;

/// Re-exports the `wslpluginapi_sys` crate as `sys` when the `sys` feature is enabled.
#[cfg(feature = "sys")]
pub use wslpluginapi_sys as sys;

pub use session_id::{HasSessionId, SessionID};
pub use user_distribution_id::UserDistributionID;
