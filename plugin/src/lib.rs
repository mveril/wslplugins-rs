use chrono::Local;
use etc_os_release::OsRelease;
use fern::{log_file, Dispatch};
use log::{info, warn, LevelFilter};
use log_instrument::instrument;
use plugin::{Result, WSLPluginV1};
use std::{env, io::Read};
use windows::{
    core::{Error as WinError, Result as WinResult, GUID},
    Win32::Foundation::E_FAIL,
};
use wslplugins_rs::wsl_user_configuration::bitflags::WSLUserConfigurationFlags;
use wslplugins_rs::*;

pub(crate) struct Plugin {
    context: &'static WSLContext,
}

fn setup_logging() -> WinResult<()> {
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
        .map_err(|_| WinError::from(E_FAIL))?;
    info!("Logging configured: {:}", log_level);
    Ok(())
}
#[wsl_plugin_v1(2, 1, 2)]
impl WSLPluginV1 for Plugin {
    fn try_new(context: &'static WSLContext) -> WinResult<Self> {
        setup_logging()?;
        let plugin = Plugin { context };
        info!("Plugin created");
        Ok(plugin)
    }

    #[instrument]
    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        user_settings: &WSLVmCreationSettings,
    ) -> Result<()> {
        let flags: WSLUserConfigurationFlags = user_settings.custom_configuration_flags().into();
        info!("User configuration {:?}", flags);

        let ver_args = ["/bin/cat", "/proc/version"];
        match self
            .context
            .api
            .execute_binary(session, ver_args[0], &ver_args)
        {
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
            // Use unknow if init_pid not available
            distribution.init_pid().map(|res| res.to_string()).unwrap_or("Unknow".to_string())
        );
        self.log_os_release(session, Some(distribution.id()));
        Ok(())
    }

    #[instrument]
    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> WinResult<()> {
        info!("VM Stopping. SessionId={:?}", session.id());
        Ok(())
    }

    #[instrument]
    fn on_distribution_stopping(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> WinResult<()> {
        info!(
            "Distribution Stopping. SessionId={}, Id={:?} name={}, package={}, PidNs={}, InitPid={}",
            session.id(),
            distribution.id(),
            distribution.name().to_string_lossy(),
            distribution.package_family_name().unwrap_or_default().to_string_lossy(),
            distribution.pid_namespace(),
            // Use unknow if init_pid not available
            distribution.init_pid().map(|res| res.to_string()).unwrap_or("Unknow".to_string())
        );
        Ok(())
    }
}

impl Plugin {
    fn log_os_release(&self, session: &WSLSessionInformation, distro_id: Option<&GUID>) {
        let args: [&str; 2] = ["/bin/cat", "/etc/os-release"];
        let tcp_stream: std::result::Result<std::net::TcpStream, api::Error> = match distro_id {
            Some(dist_id) => self
                .context
                .api
                .execute_binary_in_distribution(session, dist_id, args[0], &args),
            None => self
                .context
                .api
                .execute_binary(session, args[0], &args)
                .map_err(Into::into),
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
