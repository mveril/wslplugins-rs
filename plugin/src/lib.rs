use chrono::Local;
use etc_os_release::OsRelease;
use fern::{log_file, Dispatch};
use log::{info, warn, LevelFilter};
use log_instrument::instrument;
use std::{env, io::Read};
use windows::{
    core::{Error, Result, GUID},
    Win32::Foundation::E_FAIL,
};
use wslplugins_rs::*;

pub(crate) struct Plugin<'a> {
    api: ApiV1<'a>,
}

fn setup_logging() -> Result<()> {
    let log_level = env::var("RUST_WSL_LOGLEVEL")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(LevelFilter::Info);

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
        .map_err(|_| Error::from(E_FAIL))?;
    info!("Logging configured: {:}", log_level);
    Ok(())
}
#[wsl_plugin_v1(2, 0, 5)]
impl<'a> WSLPluginV1<'a> for Plugin<'a> {
    fn try_new(api: ApiV1<'a>) -> Result<Self> {
        setup_logging()?;
        let plugin = Plugin { api };
        info!("Plugin created");
        Ok(plugin)
    }

    #[instrument]
    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        user_settings: &WSLVmCreationSettings,
    ) -> Result<()> {
        info!(
            "User configuration {:?}",
            user_settings.custom_configuration_flags()
        );

        let ver_args = ["/bin/cat", "/proc/version"];
        match self.api.execute_binary(session, &ver_args[0], &ver_args) {
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
        let ver_args = ["/bin/cat", "/proc/version"];
        match self.api.execute_binary(session, &ver_args[0], &ver_args) {
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
        self.log_os_release(session, None);
        Ok(())
    }

    #[instrument]
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
        self.log_os_release(session, Some(distribution.id()));
        Ok(())
    }

    #[instrument]
    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> Result<()> {
        info!("VM Stopping. SessionId={:?}", session.id());
        Ok(())
    }

    #[instrument]
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

impl Plugin<'_> {
    fn log_os_release(&self, session: &WSLSessionInformation, distro_id: Option<&GUID>) {
        let args: [&str; 2] = ["/bin/cat", "/etc/os-release"];
        let tcp_stream = match distro_id {
            Some(dist_id) => self
                .api
                .execute_binary_in_distribution(session, dist_id, &args[0], &args),
            None => self.api.execute_binary(session, &args[0], &args),
        };
        let result = tcp_stream;
        match result {
            Ok(stream) => match OsRelease::from_reader(stream) {
                Ok(release) => {
                    if let Some(version) = release.version() {
                        info!("{}: ({})", release.name(), version)
                    } else {
                        info!("{}", release.name())
                    }
                }
                Err(err) => warn!("{}", err),
            },
            Err(err) => {
                warn!("Error on binary execution: {}", err)
            }
        };
    }
}
