#![doc = include_str!("../README.md")]

use std::ffi::OsString;
use std::{env, fs::OpenOptions, panic};
use tracing::{error, info, warn};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use windows::Win32::Foundation::{E_ACCESSDENIED, E_FAIL};
use wslplugins_rs::prelude::*;

fn setup_logging() -> WinResult<()> {
    // Read log level from environment first from RUST_WSL_LOGLEVEL
    let log_level = EnvFilter::try_from_env("RUST_WSL_LOGLEVEL")
        // else try default RUST_LOG
        .or_else(|_| EnvFilter::try_from_default_env())
        // fallback default if both fail
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Read log path from environment, default to C:\wsl-plugin.log
    let log_path = env::var("RUST_WSL_LOG_PATH")
        .unwrap_or_else(|_| "C:\\wsl-unpackaged-distro-blacklist.log".to_string());

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

#[derive(Debug)]
pub(crate) struct Plugin;

#[wsl_plugin_v1(2, 1, 2)]
impl WSLPluginV1 for Plugin {
    fn try_new(_context: &'static WSLContext) -> WinResult<Self> {
        setup_logging()?;
        info!("Plugin created");
        Ok(Self)
    }

    fn on_distribution_started(
        &self,
        _session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> PluginResult<()> {
        #[allow(
            clippy::option_if_let_else,
            reason = "Improve readability by using if let"
        )]
        if let Some(package_familly_name) = distribution.package_family_name() {
            info!(
                "Distribution {} started with package family name {:}",
                distribution.name().display(),
                package_familly_name.display()
            );
            Ok(())
        } else {
            let mut msg = OsString::from("The WSL distribution `");
            msg.push(distribution.name());
            msg.push("` is not allowed by your organization because it is not packaged.");
            warn!("{}", msg.display());
            Err(PluginError::with_message(E_ACCESSDENIED, &msg))
        }
    }
}
