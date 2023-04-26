use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MinistoreConfig {
    pub log: LogConfig,
    pub devices: DeviceConfig,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceConfig {
    pub use_fake: bool,
    pub list: Vec<String>,
    pub device_size: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum RunMode {
    Development,
    Production,
    /// Custom run mode has test name. There should be a config file in config folder named with test name
    Custom(String),
}
impl std::fmt::Display for RunMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            RunMode::Development => write!(f, "development"),
            RunMode::Production => write!(f, "production"),
            RunMode::Custom(mode) => write!(f, "{}", mode),
        }
    }
}

pub fn get_config(run_mode: &RunMode) -> Result<MinistoreConfig, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn cofniguring_with_development_config_file_should_success() {
        let config = get_config(&RunMode::Development).expect("Failed to get config");

        assert_eq!(config.log.level, "debug");
        assert_eq!(config.devices.use_fake, true);
    }

    #[test]
    fn cofniguring_with_production_config_file_should_success() {
        let config = get_config(&RunMode::Production).expect("Failed to get config");

        assert_eq!(config.log.level, "info");
        assert_eq!(config.devices.use_fake, false);
    }

    #[test]
    fn configuring_fake_devices_without_size_list_should_fail() {
        let test_type_name = "configuring_fake_devices_without_size_list_should_fail";

        let test_file_name = format!("config/{}.toml", test_type_name);
        std::panic::set_hook(Box::new(move |_| {
            let path = std::path::Path::new(&test_file_name);
            if path.try_exists().unwrap() {
                std::fs::remove_file(&test_file_name).expect("Failed to remove test file");
            }
        }));
        let test_config_str = r#"[log]
level = "debug"

[devices]
use_fake = true
list = [
    "fake_device_00.bin",
    "fake_device_01.bin",
    "fake_device_02.bin",
]
device_size = []
        "#;

        let mut file = std::fs::File::create(format!("config/{}.toml", test_type_name))
            .expect("Failed to create test file");
        file.write(test_config_str.as_bytes())
            .expect("Failed to write test config file");

        let config = get_config(&RunMode::Custom(test_type_name.to_string()));
        assert_eq!(config.is_err(), true);

        std::fs::remove_file(format!("config/{}.toml", test_type_name))
            .expect("Failed to remove test file");
    }
}
