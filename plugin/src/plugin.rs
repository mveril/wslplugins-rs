use std::{fs::OpenOptions, io::{BufWriter, Write, Read}, cell::RefCell};
use chrono::Local;
use wslplugins_rs::*;
use windows::core::Result;

pub(crate) struct Plugin {
    api: ApiV1,
    log_file: RefCell<BufWriter<std::fs::File>>,
}

impl Plugin {
    fn log_message(&self, message: &str) {
        let log_entry = format!(
            "{} {}\n",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            message
        );
        let mut log_file = self.log_file.borrow_mut();
        log_file.write_all(log_entry.as_bytes()).expect("Unable to write to log file");
        let _ = log_file.flush();
    }

    fn new(api: ApiV1) -> Self {
        let log_file_path = "C:\\wsl-plugin.log";
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .expect("Unable to open log file");

        let log_file = RefCell::new(BufWriter::new(file));
        let plugin = Plugin { api, log_file };
        plugin.log_message("Plugin created");
        plugin
    }
}

impl WSLPluginV1 for Plugin {
    fn try_new(api: ApiV1) -> Result<Self> {
        Ok(Plugin::new(api))
    }

    fn on_vm_started(&self, session: &WSLSessionInformation, user_settings: &WSLVmCreationSettings) -> Result<()> {
        self.log_message(&format!("User configuration {:?}", user_settings.custom_configuration_flags()));

        let args = vec!["/bin/cat", "/proc/version"];
        let result = self.api.execute_binary(session, args[0], &args);
        match result {
            Ok(mut stream) => {
                let mut buf = String::new();
                if stream.read_to_string(&mut buf).is_ok_and(|size| size != 0) {
                    self.log_message(&format!("Kernel version info: {}", buf.trim()));
                } else {
                    self.log_message("No version found");
                }
            },
            Err(err) => self.log_message(&format!("Error on {}: {}", stringify!(on_vm_started), err)),
        };
        Ok(())
    }

    fn on_distribution_started(&self, session: &WSLSessionInformation, distribution: &DistributionInformation) -> Result<()> {
        self.log_message(&format!(
            "Distribution started. Sessionid= {:}, Id={:?} Name={:}, Package={}, PidNs={}, InitPid={}",
            session.id(),
            distribution.id(),
            distribution.name().to_string_lossy(),
            distribution.package_family_name().unwrap_or_default().to_string_lossy(),
            distribution.pid_namespace(),
            distribution.init_pid()
        ));
        Ok(())
    }

    fn on_vm_stopping(&self, session: &WSLSessionInformation) -> Result<()> {
        self.log_message(&format!("VM Stopping. SessionId={:?}", session.id()));
        Ok(())
    }

    fn on_distribution_stopping(&self, session: &WSLSessionInformation, distribution: &DistributionInformation) -> Result<()> {
        self.log_message(&format!(
            "Distribution Stopping. SessionId={}, Id={:?} name={}, package={}, PidNs={}, InitPid={}",
            session.id(),
            distribution.id(),
            distribution.name().to_string_lossy(),
            distribution.package_family_name().unwrap_or_default().to_string_lossy(),
            distribution.pid_namespace(),
            distribution.init_pid()
        ));
        Ok(())
    }
}
