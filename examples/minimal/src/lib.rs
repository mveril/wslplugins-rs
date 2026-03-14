//! Sample WSL plugin implemented in Rust.
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use windows::Win32::Foundation::{E_ABORT, E_FAIL};
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
            .write(true)
            .truncate(true)
            .open("C:\\wsl-plugin-demo.txt")
            .map_err(|_| WinError::from(E_ABORT))?;
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
        writeln!(
            &self.log_file,
            "VM created. SessionId={}, CustomConfigurationFlags={}",
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
            Err(error) => {
                writeln!(&self.log_file, "Failed to create process, {}", error.code())
                    .map_err(|_| WinError::from(E_FAIL))?;
                Err(error)?;
            }
            Ok(mut stream) => {
                let mut buffer = String::new();
                stream
                    .read_to_string(&mut buffer)
                    .map_err(|_| WinError::from(E_FAIL))?;

                if buffer.is_empty() {
                    writeln!(&self.log_file, "cat /proc/version returned no output")
                        .map_err(|_| WinError::from(E_FAIL))?;
                    return Ok(());
                }

                if buffer.ends_with('\n') {
                    buffer.pop();
                }
                writeln!(&self.log_file, "Kernel version info: {buffer}")
                    .map_err(|_| WinError::from(E_FAIL))?;
            }
        }
        Ok(())
    }
    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> WinResult<()> {
        writeln!(&self.log_file, "VM Stopping. SessionId={}", session.id())?;
        Ok(())
    }
    fn on_distribution_started(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> PluginResult<()> {
        let init_pid = distribution.init_pid()?;
        writeln!(
            &self.log_file,
            "Distribution started. Sessionid= {}, Name={}, Package={}, PidNs={}, InitPid={}",
            session.id(),
            distribution.name().to_string_lossy(),
            distribution
                .package_family_name()
                .unwrap_or_default()
                .display(),
            distribution.pid_namespace(),
            init_pid
        )
        .map_err(|_| WinError::from(E_FAIL))?;
        Ok(())
    }

    fn on_distribution_stopping(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> WinResult<()> {
        let init_pid = distribution.init_pid()?;
        writeln!(
            &self.log_file,
            "Distribution Stopping. SessionId={}, name={}, package={}, PidNs={}, InitPid={}",
            session.id(),
            distribution.name().to_string_lossy(),
            distribution
                .package_family_name()
                .unwrap_or_default()
                .display(),
            distribution.pid_namespace(),
            init_pid
        )
        .map_err(|_| WinError::from(E_FAIL))?;
        Ok(())
    }

    fn on_distribution_registered(
        &self,
        session: &WSLSessionInformation,
        distribution: &OfflineDistributionInformation,
    ) -> WinResult<()> {
        write!(
            &self.log_file,
            "Distribution registeredd. SessionId={}, name={}, package={}",
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
        write!(
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
