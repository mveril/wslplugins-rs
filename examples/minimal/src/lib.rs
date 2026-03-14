//! Sample WSL plugin implemented in Rust.
use std::borrow::Cow;
use std::fs::File;
use std::io::prelude::*;
use std::{fs::OpenOptions, io::Read};
use windows::Win32::Foundation::E_FAIL;
use wslplugins_rs::prelude::*;

#[derive(Debug)]
pub(crate) struct Plugin {
    context: &'static WSLContext,
    log_file: File,
}

#[wsl_plugin_v1(2, 1, 3)]
impl WSLPluginV1 for Plugin {
    fn try_new(context: &'static WSLContext) -> WinResult<Self> {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("C:\\wsl-plugin-demo.txt")
            .map_err(|_| WinError::from(E_FAIL))?;
        writeln!(
            &log_file,
            "Plugin loaded. WSL version: {}",
            context.api.version()
        )?;
        let plugin = Self { context, log_file };
        Ok(plugin)
    }

    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        user_settings: &WSLVmCreationSettings,
    ) -> PluginResult<()> {
        #[allow(clippy::use_debug)]
        writeln!(
            &self.log_file,
            "VM created. SessionId={}, CustomConfigurationFlags={:?}",
            session.id(),
            user_settings.custom_configuration_flags()
        )
        .map_err(|_| WinError::from(E_FAIL))?;

        // Launch cat /proc/version to get the VM's kernel version
        match self
            .context
            .api
            .new_command(session.id(), "/bin/cat")
            .with_arg("/proc/version")
            .execute()
        {
            Err(e) => {
                writeln!(&self.log_file, "Failed to execute command: {e}")
                    .map_err(|_| WinError::from(E_FAIL))?;
            }
            Ok(mut stream) => {
                let mut buffer = String::new();
                stream
                    .read_to_string(&mut buffer)
                    .map_err(|_| WinError::from(E_FAIL))?;
                writeln!(&self.log_file, "Kernel version info: {}", buffer.trim())
                    .map_err(|_| WinError::from(E_FAIL))?;
            }
        }
        Ok(())
    }
    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> WinResult<()> {
        writeln!(&self.log_file, "VM stopping. SessionId={}", session.id())?;
        Ok(())
    }
    fn on_distribution_started(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> PluginResult<()> {
        writeln!(
            &self.log_file,
            "Distribution started. SessionId={}, name={}, package={}, InitPid={}",
            session.id(),
            distribution.name().to_string_lossy(),
            distribution
                .package_family_name()
                .unwrap_or_default()
                .display(),
            distribution
                .init_pid()
                .map_or(Cow::Borrowed(""), |pid| Cow::Owned(pid.to_string()))
        )
        .map_err(|_| WinError::from(E_FAIL))?;
        Ok(())
    }

    fn on_distribution_stopping(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> WinResult<()> {
        writeln!(
            &self.log_file,
            "Distribution Stopping. SessionId={}, name={}, package={}, InitPid={}",
            session.id(),
            distribution.name().to_string_lossy(),
            distribution
                .package_family_name()
                .unwrap_or_default()
                .display(),
            distribution
                .init_pid()
                .map_or(Cow::Borrowed(""), |pid| Cow::Owned(pid.to_string()))
        )
        .map_err(|_| WinError::from(E_FAIL))?;
        Ok(())
    }

    fn on_distribution_registered(
        &self,
        session: &WSLSessionInformation,
        distribution: &OfflineDistributionInformation,
    ) -> WinResult<()> {
        writeln!(
            &self.log_file,
            "Distribution registered. SessionId={}, name={}, package={}",
            session.id(),
            distribution.name().to_string_lossy(),
            distribution
                .package_family_name()
                .unwrap_or_default()
                .display()
        )
        .map_err(|_| WinError::from(E_FAIL))?;
        Ok(())
    }

    fn on_distribution_unregistered(
        &self,
        session: &WSLSessionInformation,
        distribution: &OfflineDistributionInformation,
    ) -> WinResult<()> {
        writeln!(
            &self.log_file,
            "Distribution unregistered. SessionId={}, name={}, package={}",
            session.id(),
            distribution.name().to_string_lossy(),
            distribution
                .package_family_name()
                .unwrap_or_default()
                .display()
        )
        .map_err(|_| WinError::from(E_FAIL))?;
        Ok(())
    }
}
