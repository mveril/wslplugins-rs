//! Sample WSL plugin implemented in Rust.
use etc_os_release::OsRelease;
use plugin::{Result, WSLPluginV1};
use std::{env, fs::OpenOptions, io::Read, panic};
use tracing::{error, info, instrument, warn};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use windows::{
    core::{Error as WinError, Result as WinResult, GUID},
    Win32::Foundation::E_FAIL,
};
use wslplugins_rs::wsl_user_configuration::bitflags::WSLUserConfigurationFlags;
use wslplugins_rs::*;

#[derive(Debug)]
pub(crate) struct Plugin {
    context: &'static WSLContext,
}

fn setup_logging() -> WinResult<()> {
    // Read log level from environment first from RUST_WSL_LOGLEVEL
    let log_level = EnvFilter::try_from_env("RUST_WSL_LOGLEVEL")
        // else try default RUST_LOG
        .or_else(|_| EnvFilter::try_from_default_env())
        // fallback default if both fail
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Read log path from environment, default to C:\wsl-plugin.log
    let log_path =
        env::var("RUST_WSL_LOG_PATH").unwrap_or_else(|_| "C:\\wsl-plugin.log".to_string());

    // Open (or create) the log file in append mode
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|_| WinError::from(E_FAIL))?;

    // Non-blocking writer so logging does not block the main thread
    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    // Leak the guard so it lives for the whole process lifetime
    Box::leak(Box::new(guard));

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_writer(non_blocking)
        .with_ansi(false) // log file, no ANSI colors
        .with_span_events(FmtSpan::ACTIVE)
        .try_init()
        .map_err(|_| WinError::from(E_FAIL))?;

    info!("Logging configured to path {}", log_path);
    panic::set_hook(Box::new(|info| {
        // This will be called for *every* panic before unwinding
        error!("panic: {info}");
    }));
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

    #[instrument(level = "trace")]
    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        user_settings: &WSLVmCreationSettings,
    ) -> Result<()> {
        let flags: WSLUserConfigurationFlags = user_settings.custom_configuration_flags().into();
        info!("User configuration {flags:?}");

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

    #[instrument(level = "trace")]
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

    #[instrument(level = "trace")]
    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> WinResult<()> {
        info!("VM Stopping. SessionId={:?}", session.id());
        Ok(())
    }

    #[instrument(level = "trace")]
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
    fn log_os_release(&self, session: &WSLSessionInformation, distro_id: Option<GUID>) {
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
                Err(err) => warn!("{err}"),
            },
            Err(err) => {
                warn!("Error on binary execution: {err}")
            }
        };
    }
}
