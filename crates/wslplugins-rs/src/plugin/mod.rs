//! # WSL Plugin Framework
//!
//! This module serves as the main entry point for creating WSL plugins using Rust.
//! It provides error handling, utilities, and the WSL Plugin v1 interface for seamless integration
//! with the Windows Subsystem for Linux (WSL) plugin system.

/// Error handling utilities for WSL plugins.
///
/// This module defines custom error types and result aliases to simplify and standardize
/// error handling throughout the WSL plugin framework.
pub mod error;

/// The WSL Plugin v1 interface.
///
/// This module provides the implementation and traits required to define and interact with
/// WSL plugins compatible with the Plugin API version 1.
pub mod wsl_plugin_v1;

/// Utility functions for WSL plugin development.
///
/// This module includes helper functions and utilities to facilitate common tasks in WSL plugin creation.
pub mod utils;

/// The primary error type for the WSL plugin framework.
///
/// Refer to [`error::Error`] for more details.
pub use error::Error;

/// A specialized result type for operations within the WSL plugin framework.
///
/// Refer to [`error::Result`] for more details.
pub use error::Result;

/// The core interface for WSL plugins.
///
/// Refer to [`wsl_plugin_v1::WSLPluginV1`] for more details.
pub use wsl_plugin_v1::WSLPluginV1;

/// A utility function to create a plugin with a specified required version.
///
/// Refer to [`utils::create_plugin_with_required_version`] for more details.
pub use utils::{
    create_plugin_with_required_capabilities, create_plugin_with_required_capability,
    create_plugin_with_required_version,
};
