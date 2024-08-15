use crate::{
    distribution_information::DistributionInformation,
    offline_distribution_information::OfflineDistributionInformation,
    wsl_session_information::WSLSessionInformation,
    wsl_vm_creation_settings::WSLVmCreationSettings, ApiV1,
};
use std::marker::Sized;
use windows::core::Result;

/// Trait defining synchronous notifications sent to the plugin.
pub trait WSLPluginV1: Sized {
    fn try_new(api: ApiV1) -> Result<Self>;

    /// Called when the VM has started.
    #[allow(unused_variables)]
    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        user_settings: &WSLVmCreationSettings,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when the VM is about to stop.
    #[allow(unused_variables)]
    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> Result<()> {
        Ok(())
    }

    /// Called when a distribution has started.
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
    /// Note: It's possible that stopping a distribution fails (for instance, if a file is in use).
    /// In this case, this notification might be called multiple times for the same distribution.
    #[allow(unused_variables)]
    fn on_distribution_stopping(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when a distribution is registered or unregistered.
    ///
    /// Returning failure will NOT cause the operation to fail.
    /// Introduced in 2.1.2
    #[allow(unused_variables)]
    fn on_distribution_registered(
        &self,
        session: &WSLSessionInformation,
        distribution: &OfflineDistributionInformation,
    ) -> Result<()> {
        Ok(())
    }
    /// Called when a distribution is registered or unregisteed.
    ///
    /// Returning failure will NOT cause the operation to fail.
    /// Introduced in 2.1.2
    #[allow(unused_variables)]
    fn on_distribution_unregistered(
        &self,
        session: &WSLSessionInformation,
        distribution: &OfflineDistributionInformation,
    ) -> Result<()> {
        Ok(())
    }
}
