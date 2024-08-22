use chrono::Local;
use fern::{log_file, Dispatch};
use log::{info, warn, LevelFilter};
use std::{env, io::Read};
use windows::{core::Result, Win32::Foundation::E_FAIL};
use wslplugins_rs::*;

pub(crate) struct Plugin<'a> {
    api: ApiV1<'a>,
}

fn setup_logging() -> Result<()> {
    let log_level = match env::var("RUST_WSL_LOGLEVEL")
        .ok()
        .and_then(|val| val.parse().ok())
    {
        Some(level) => level,
        None => LevelFilter::Info,
    };

    let log_path =
        env::var("RUST_WSL_LOG_PATH").unwrap_or_else(|_| "C:\\wsl-plugin.log".to_string());

    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(log_file(log_path)?)
        .apply()
        .map_err(|_| E_FAIL.into())
}

impl<'a> WSLPluginV1<'a> for Plugin<'a> {
    fn try_new(api: ApiV1<'a>) -> Result<Self> {
        setup_logging()?;
        let plugin = Plugin { api };
        info!("Plugin created");
        Ok(plugin)
    }

    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        user_settings: &WSLVmCreationSettings,
    ) -> Result<()> {
        info!(
            "User configuration {:?}",
            user_settings.custom_configuration_flags()
        );

        let args = vec!["/bin/cat", "/proc/version"];
        let result = self.api.execute_binary(session, args[0], &args);
        match result {
            Ok(mut stream) => {
                let mut buf = String::new();
                if stream.read_to_string(&mut buf).is_ok_and(|size| size != 0) {
                    info!("Kernel version info: {}", buf.trim());
                } else {
                    warn!("No version found");
                }
            }
            Err(err) => {
                warn!(
                    "Error on binary execution {}: {}",
                    stringify!(on_vm_started),
                    err
                )
            }
        };
        Ok(())
    }

    fn on_distribution_started(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> Result<()> {
        info!(
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
        info!("VM Stopping. SessionId={:?}", session.id());
        Ok(())
    }

    fn on_distribution_stopping(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> Result<()> {
        info!(
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
