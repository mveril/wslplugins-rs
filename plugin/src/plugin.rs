use chrono::Local;
use std::{
    cell::RefCell,
    fs::OpenOptions,
    io::{self, BufWriter, Read, Write},
};
use windows::core::Result;
use wslplugins_rs::*;

pub(crate) struct Plugin {
    api: ApiV1,
    log_file: RefCell<BufWriter<std::fs::File>>,
}

impl Plugin {
    fn log_message(&self, message: &str) -> io::Result<()> {
        let mut log_file = self.log_file.borrow_mut();
        log_file.write_fmt(format_args!(
            "{} {}\n",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            message
        ))?;
        log_file.flush()?;
        Ok(())
    }
}

impl WSLPluginV1 for Plugin {
    fn try_new(api: ApiV1) -> Result<Self> {
        let log_file_path = "C:\\wsl-plugin.log";
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)?;

        let log_file = RefCell::new(BufWriter::new(file));
        let plugin = Plugin { api, log_file };
        plugin.log_message("Plugin created")?;
        Ok(plugin)
    }

    fn on_vm_started(
        &self,
        session: &WSLSessionInformation,
        user_settings: &WSLVmCreationSettings,
    ) -> Result<()> {
        self.log_message(&format!(
            "User configuration {:?}",
            user_settings.custom_configuration_flags()
        ))?;

        let args = vec!["/bin/cat", "/proc/version"];
        let result = self.api.execute_binary(session, args[0], &args);
        match result {
            Ok(mut stream) => {
                let mut buf = String::new();
                if stream.read_to_string(&mut buf).is_ok_and(|size| size != 0) {
                    self.log_message(&format!("Kernel version info: {}", buf.trim()))?;
                } else {
                    self.log_message("No version found")?;
                }
            }
            Err(err) => {
                self.log_message(&format!("Error on {}: {}", stringify!(on_vm_started), err))?
            }
        };
        Ok(())
    }

    fn on_distribution_started(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> Result<()> {
        self.log_message(&format!(
            "Distribution started. Sessionid= {:}, Id={:?} Name={:}, Package={}, PidNs={}, InitPid={}",
            session.id(),
            distribution.id(),
            distribution.name().to_string_lossy(),
            distribution.package_family_name().unwrap_or_default().to_string_lossy(),
            distribution.pid_namespace(),
            distribution.init_pid()
        ))?;
        Ok(())
    }

    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> Result<()> {
        self.log_message(&format!("VM Stopping. SessionId={:?}", session.id()))?;
        Ok(())
    }

    fn on_distribution_stopping(
        &self,
        session: &WSLSessionInformation,
        distribution: &DistributionInformation,
    ) -> Result<()> {
        self.log_message(&format!(
            "Distribution Stopping. SessionId={}, Id={:?} name={}, package={}, PidNs={}, InitPid={}",
            session.id(),
            distribution.id(),
            distribution.name().to_string_lossy(),
            distribution.package_family_name().unwrap_or_default().to_string_lossy(),
            distribution.pid_namespace(),
            distribution.init_pid()
        ))?;
        Ok(())
    }
}
