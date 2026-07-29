//! # WSL Plugin v1 Trait
//!
//! This module defines the `WSLPluginV1` trait, which provides a framework for handling
//! synchronous notifications sent to a WSL plugin. The trait defines lifecycle events
//! for managing the state of the WSL VM, distributions, and related settings.

#[cfg(doc)]
use super::error::Error;
use super::error::Result;
#[cfg(feature = "experimental-wslc")]
use crate::api::{
    WSLCContainerInformation, WSLCImageInformation, WSLCObjectId, WSLCSessionInformation,
};
#[cfg(doc)]
use crate::WSLVersionCapability;
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
/// Plugin types are usually registered with the `wsl_plugin_v1` attribute macro
/// re-exported by `wslplugins-rs` when the `macro` feature is enabled. The macro
/// generates the exported WSL entry point, initializes [`WSLContext`], creates one
/// plugin instance through [`WSLPluginV1::try_new`], and wires the trait methods
/// that are implemented by the plugin.
///
/// The macro accepts either no argument, an explicit minimum API version, or one
/// or more `WSLVersionCapability` values. Use no argument for plugins that only
/// need the base entry point. Use capabilities when the plugin depends on named
/// API features such as distribution registration hooks.
///
/// # Requirements
/// - The trait is `Sized` and `Sync`, ensuring safe concurrent usage and instantiation.
///
/// # Example
/// ```rust,ignore
/// use wslplugins_rs::prelude::*;
/// use wslplugins_rs::{WSLSessionInformation, WSLVmCreationSettings};
/// use wslplugins_rs::windows_core::Result as WinResult;
///
/// struct MyPlugin {
///     context: &'static WSLContext,
/// }
///
/// #[wsl_plugin_v1]
/// impl WSLPluginV1 for MyPlugin {
///     fn try_new(context: &'static WSLContext) -> WinResult<Self> {
///         Ok(Self { context })
///     }
///
///     fn on_vm_started(
///         &self,
///         session: &WSLSessionInformation,
///         user_settings: &WSLVmCreationSettings,
///     ) -> Result<()> {
///         let _ = (session, user_settings);
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
    /// - This hook is wired only when the host API supports
    ///   [`WSLVersionCapability::DistributionRegisteredHook`] (`2.1.2`).
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
    /// - This hook is wired only when the host API supports
    ///   [`WSLVersionCapability::DistributionUnregisteredHook`] (`2.1.2`).
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

    /// Called when a WSLC session has been created.
    ///
    /// Returning an error prevents the session creation.
    ///
    /// # Errors
    ///
    /// Return an error to reject session creation.
    #[cfg(feature = "experimental-wslc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental-wslc")))]
    #[inline]
    fn on_session_created(&self, _session: &WSLCSessionInformation) -> WinResult<()> {
        Ok(())
    }

    /// Called when a WSLC session is about to stop.
    ///
    /// # Errors
    ///
    /// Errors are reported to the host, which may ignore them while stopping.
    #[cfg(feature = "experimental-wslc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental-wslc")))]
    #[inline]
    fn on_session_stopping(&self, _session: &WSLCSessionInformation) -> WinResult<()> {
        Ok(())
    }

    /// Called when a WSLC container starts.
    ///
    /// Returning an error prevents the container creation.
    ///
    /// # Errors
    ///
    /// Return an error to reject container creation.
    #[cfg(feature = "experimental-wslc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental-wslc")))]
    #[inline]
    fn container_started(
        &self,
        _session: &WSLCSessionInformation,
        _container: &WSLCContainerInformation,
    ) -> WinResult<()> {
        Ok(())
    }

    /// Called when a WSLC container is about to stop.
    ///
    /// # Errors
    ///
    /// Errors are reported to the host, which may ignore them while stopping.
    #[cfg(feature = "experimental-wslc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental-wslc")))]
    #[inline]
    fn container_stopping(
        &self,
        _session: &WSLCSessionInformation,
        _container_id: &WSLCObjectId,
    ) -> WinResult<()> {
        Ok(())
    }

    /// Called when a WSLC image is pulled or imported.
    ///
    /// # Errors
    ///
    /// Errors are reported to the host, which currently ignores them.
    #[cfg(feature = "experimental-wslc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental-wslc")))]
    #[inline]
    fn image_created(
        &self,
        _session: &WSLCSessionInformation,
        _image: &WSLCImageInformation,
    ) -> WinResult<()> {
        Ok(())
    }

    /// Called when a WSLC image is deleted.
    ///
    /// # Errors
    ///
    /// Errors are reported to the host, which currently ignores them.
    #[cfg(feature = "experimental-wslc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental-wslc")))]
    #[inline]
    fn image_deleted(
        &self,
        _session: &WSLCSessionInformation,
        _image_id: &WSLCObjectId,
    ) -> WinResult<()> {
        Ok(())
    }
}
