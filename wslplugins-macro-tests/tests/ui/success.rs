use windows::core::Result;
use wslplugins_rs::*;

pub(crate) struct Plugin<'a> {
    api: ApiV1<'a>,
}
#[wsl_plugin_v1(1, 0, 5)]
impl<'a> WSLPluginV1<'a> for Plugin<'a> {
    fn try_new(api: ApiV1<'a>) -> Result<Self> {
        let plugin = Plugin { api };
        Ok(plugin)
    }

    fn on_vm_started(
        &self,
        _session: &WSLSessionInformation,
        user_settings: &WSLVmCreationSettings,
    ) -> Result<()> {
        println!(
            "User configuration {:?}",
            user_settings.custom_configuration_flags()
        );
        Ok(())
    }

    fn on_distribution_started(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> Result<()> {
        println!(
            "Distribution started. Sessionid= {:}, Id={:?} Name={:}, Package={}, PidNs={}, InitPid={}",
            session.id(),
            distribution.id(),
            distribution.name().to_string_lossy(),
            distribution.package_family_name().unwrap_or_default().to_string_lossy(),
            distribution.pid_namespace(),
            distribution.init_pid()
        );
        Ok(())
    }

    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> Result<()> {
        println!("VM Stopping. SessionId={:?}", session.id());
        Ok(())
    }

    fn on_distribution_stopping(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> Result<()> {
        println!(
            "Distribution Stopping. SessionId={}, Id={:?} name={}, package={}, PidNs={}, InitPid={}",
            session.id(),
            distribution.id(),
            distribution.name().to_string_lossy(),
            distribution.package_family_name().unwrap_or_default().to_string_lossy(),
            distribution.pid_namespace(),
            distribution.init_pid()
        );
        Ok(())
    }
}

// trybuild need mains
fn main() {}
