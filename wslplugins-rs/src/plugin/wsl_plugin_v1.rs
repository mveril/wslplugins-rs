//! # WSL Plugin v1 Trait
//!
//! This module defines the `WSLPluginV1` trait, which provides a framework for handling
//! synchronous notifications sent to a WSL plugin. The trait defines lifecycle events
//! for managing the state of the WSL VM, distributions, and related settings.

use super::error::Result;
use crate::WSLContext;
use crate::{
    distribution_information::DistributionInformation,
    offline_distribution_information::OfflineDistributionInformation,
    wsl_session_information::WSLSessionInformation,
    wsl_vm_creation_settings::WSLVmCreationSettings,
};
use std::marker::Sized;
use windows::core::Result as WinResult;

/// Trait defining synchronous notifications sent to the plugin.
///
/// Implementors of this trait must provide methods for responding to key lifecycle events
/// in the WSL plugin system. Each method corresponds to a specific event, such as the start
/// or stop of a WSL VM or distribution.
///
/// # Requirements
/// - The trait is `Sized` and `Sync`, ensuring safe concurrent usage and instantiation.
///
/// # Example
/// ```rust
/// use wslplugins_rs::{plugin::{WSLPluginV1, Result}, WSLContext, WSLSessionInformation, WSLVmCreationSettings};
/// use windows::core::Result as WinResult;
///
/// struct MyPlugin;
///
/// impl WSLPluginV1 for MyPlugin {
///     fn try_new(context: &'static WSLContext) -> WinResult<Self> {
///         Ok(MyPlugin)
///     }
///
///     fn on_vm_started(
///         &self,
///         session: &WSLSessionInformation,
///         user_settings: &WSLVmCreationSettings,
///     ) -> Result<()> {
///         println!("VM started");
///         Ok(())
///     }
/// }
/// ```
pub trait WSLPluginV1: Sized + Sync {
    /// Attempts to create a new instance of the plugin.
    ///
    /// # Arguments
    /// - `context`: A reference to the `WSLContext` providing access to the plugin API.
    ///
    /// # Returns
    /// - `Ok(Self)`: If the plugin was successfully initialized.
    /// - `Err(WinError)`: If initialization fails.
    fn try_new(context: &'static WSLContext) -> WinResult<Self>;

    /// Called when the VM has started.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    /// - `user_settings`: Custom user settings for the VM creation.
    ///
    /// # Returns
    /// - `Ok(())`: If the plugin successfully handled the event.
    /// - `Err(`Error`)`: If the event handling failed.
    #[allow(unused_variables)]
    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        user_settings: &WSLVmCreationSettings,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when the VM is about to stop.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    ///
    /// # Returns
    /// - `Ok(())`: If the plugin successfully handled the event.
    /// - `Err(WinError)`: If the event handling failed.
    #[allow(unused_variables)]
    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> WinResult<()> {
        Ok(())
    }

    /// Called when a distribution has started.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    /// - `distribution`: Information about the distribution.
    ///
    /// # Returns
    /// - `Ok(())`: If the plugin successfully handled the event.
    /// - `Err(Error)`: If the event handling failed.
    #[allow(unused_variables)]
    fn on_distribution_started(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when a distribution is about to stop.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    /// - `distribution`: Information about the distribution.
    ///
    /// # Returns
    /// - `Ok(())`: If the plugin successfully handled the event.
    /// - `Err(WinError)`: If the event handling failed.
    ///
    /// # Notes
    /// - This method might be called multiple times for the same distribution if stopping fails.
    #[allow(unused_variables)]
    fn on_distribution_stopping(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> WinResult<()> {
        Ok(())
    }

    /// Called when a distribution is registered.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    /// - `distribution`: Offline information about the distribution.
    ///
    /// # Returns
    /// - `Ok(())`: If the plugin successfully handled the event.
    /// - `Err(WinError)`: If the event handling failed.
    ///
    /// # Notes
    /// - Introduced in API version 2.1.2.
    #[allow(unused_variables)]
    fn on_distribution_registered(
        &self,
        session: &WSLSessionInformation,
        distribution: &OfflineDistributionInformation,
    ) -> WinResult<()> {
        Ok(())
    }

    /// Called when a distribution is unregistered.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    /// - `distribution`: Offline information about the distribution.
    ///
    /// # Returns
    /// - `Ok(())`: If the plugin successfully handled the event.
    /// - `Err(WinError)`: If the event handling failed.
    ///
    /// # Notes
    /// - Introduced in API version 2.1.2.
    #[allow(unused_variables)]
    fn on_distribution_unregistered(
        &self,
        session: &WSLSessionInformation,
        distribution: &OfflineDistributionInformation,
    ) -> WinResult<()> {
        Ok(())
    }
}
