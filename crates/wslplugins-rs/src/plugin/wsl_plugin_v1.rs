//! # WSL Plugin v1 Trait
//!
//! This module defines the `WSLPluginV1` trait, which provides a framework for handling
//! synchronous notifications sent to a WSL plugin. The trait defines lifecycle events
//! for managing the state of the WSL VM, distributions, and related settings.

#[cfg(doc)]
use super::error::Error;
use super::error::Result;
use crate::{
    wsl_distribution_information::WSLDistributionInformation,
    wsl_offline_distribution_information::WSLOfflineDistributionInformation,
    wsl_session_information::WSLSessionInformation,
    wsl_vm_creation_settings::WSLVmCreationSettings, WSLContext,
};
use std::marker::Sized;
#[cfg(doc)]
use windows_core::Error as WinError;
use windows_core::Result as WinResult;

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
/// use wslplugins_rs::windows_core::Result as WinResult;
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
    /// # Errors
    /// - [`WinError`]: If initialization fails.
    fn try_new(context: &'static WSLContext) -> WinResult<Self>;

    /// Called when the VM has started.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    /// - `user_settings`: Custom user settings for the VM creation.
    ///
    /// # Errors
    /// - [`Error`]: If the event handling failed.
    #[expect(
        unused_variables,
        reason = "We are on a treit with default methods that return just Ok(())"
    )]
    #[inline]
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
    /// # Errors
    /// - `Errors`: If the event handling failed.
    #[expect(
        unused_variables,
        reason = "We are on a treit with default methods that return just Ok(())"
    )]
    #[inline]
    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> WinResult<()> {
        Ok(())
    }

    /// Called when a distribution has started.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    /// - `distribution`: Information about the distribution.
    ///
    /// # Errors
    /// - `Errors`: If the event handling failed.
    #[expect(
        unused_variables,
        reason = "We are on a treit with default methods that return just Ok(())"
    )]
    #[inline]
    fn on_distribution_started(
        &self,
        session: &WSLSessionInformation,
        distribution: &WSLDistributionInformation,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when a distribution is about to stop.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    /// - `distribution`: Information about the distribution.
    ///
    /// # Errors
    /// - `WinError`: If the event handling failed.
    ///
    /// # Notes
    /// - This method might be called multiple times for the same distribution if stopping fails.
    #[expect(
        unused_variables,
        reason = "We are on a treit with default methods that return just Ok(())"
    )]
    #[inline]
    fn on_distribution_stopping(
        &self,
        session: &WSLSessionInformation,
        distribution: &WSLDistributionInformation,
    ) -> WinResult<()> {
        Ok(())
    }

    /// Called when a distribution is registered.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    /// - `distribution`: Offline information about the distribution.
    ///
    /// # Errors
    /// - `WinError`: If the event handling failed.
    /// # Notes
    /// - Introduced in API version 2.1.2.
    #[expect(
        unused_variables,
        reason = "We are on a treit with default methods that return just Ok(())"
    )]
    #[inline]
    fn on_distribution_registered(
        &self,
        session: &WSLSessionInformation,
        distribution: &WSLOfflineDistributionInformation,
    ) -> WinResult<()> {
        Ok(())
    }

    /// Called when a distribution is unregistered.
    ///
    /// # Arguments
    /// - `session`: Information about the current session.
    /// - `distribution`: Offline information about the distribution.
    ///
    /// # Errors
    /// - `WinError`: If the event handling failed.
    ///
    /// # Notes
    /// - Introduced in API version 2.1.2.
    #[expect(
        unused_variables,
        reason = "We are on a treit with default methods that return just 
    Ok(())"
    )]
    #[inline]
    fn on_distribution_unregistered(
        &self,
        session: &WSLSessionInformation,
        distribution: &WSLOfflineDistributionInformation,
    ) -> WinResult<()> {
        Ok(())
    }
}
