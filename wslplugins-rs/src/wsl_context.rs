//! # WSL Context Management
//!
//! This module provides a global context for interacting with the WSL plugin API. It ensures
//! that a single instance of `WSLContext` is initialized and accessible globally during the
//! plugin's lifecycle.

use crate::api::ApiV1;
use std::sync::OnceLock;

/// A static instance of `WSLContext` initialized once during the program's lifecycle.
static CURRENT_CONTEXT: OnceLock<WSLContext> = OnceLock::new();

/// A global context for the WSL plugin API.
///
/// The `WSLContext` contains the API interface (`ApiV1`) and ensures safe, global access
/// throughout the plugin's lifecycle.
pub struct WSLContext {
    /// The API interface used for interacting with the WSL plugin API.
    pub api: &'static ApiV1,
}

impl WSLContext {
    /// Retrieves the current [WSLContext] instance, if it has been initialized.
    ///
    /// # Returns
    /// - `Some(&'static WSLContext)`: If the context has been initialized.
    /// - `None`: If the context has not yet been initialized.
    ///
    /// # Example
    /// ```rust
    /// use wslplugins_rs::WSLContext;
    ///
    /// if let Some(context) = WSLContext::get_current() {
    ///     // Use the context
    /// } else {
    ///     eprintln!("WSL context is not initialized.");
    /// }
    /// ```
    pub fn get_current() -> Option<&'static WSLContext> {
        CURRENT_CONTEXT.get()
    }

    /// Retrieves the current `WSLContext` instance or panics if it is not initialized.
    ///
    /// # Panics
    /// This function will panic if the context has not been initialized.
    ///
    /// # Returns
    /// A reference to the current `WSLContext`.
    pub fn get_current_or_panic() -> &'static WSLContext {
        Self::get_current().expect("WSL context is not initialised.")
    }

    /// Initializes the global `WSLContext` with the provided `ApiV1` instance.
    ///
    /// # Arguments
    /// - `api`: The [ApiV1] instance to associate with the context.
    ///
    /// # Returns
    /// - `Some(&'static WSLContext)`: If the context was successfully initialized.
    /// - `None`: If the context has already been initialized.
    pub fn init(api: &'static ApiV1) -> Option<&'static Self> {
        CURRENT_CONTEXT.set(WSLContext { api }).ok()?;
        CURRENT_CONTEXT.get()
    }
}
