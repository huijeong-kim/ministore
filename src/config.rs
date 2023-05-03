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

#[derive(Debug, Deserialize, Clone)]
pub struct DeviceConfig {
    pub use_fake: bool,
    pub fake_device_location: String,
    pub fake_device_type: String,
}

#[derive(Debug, PartialEq)]
pub enum RunMode {
    Development,
    Production,
    /// In test mode, configuration file for this test should be provided
    Test(String),
}
impl std::fmt::Display for RunMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            RunMode::Development => write!(f, "development"),
            RunMode::Production => write!(f, "production"),
            RunMode::Test(_) => write!(f, "Test"),
        }
    }
}
impl RunMode {
    fn get_config_file(&self) -> &str {
        match self {
            RunMode::Development => "config/development.toml",
            RunMode::Production => "config/production.toml",
            RunMode::Test(file) => file,
        }
    }
}
pub fn get_config(run_mode: &RunMode) -> Result<MinistoreConfig, String> {
    let config = Config::builder()
        .add_source(File::with_name("config/default.toml"))
        .add_source(File::with_name(run_mode.get_config_file()))
        .build()
        .map_err(|e| e.to_string())?;

    let config: MinistoreConfig = config.try_deserialize().map_err(|e| e.to_string())?;

    Ok(config)
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
    fn configuring_with_test_mode_should_success() {
        let test_config_str = r#"[log]
    level = "debug"

    [devices]
    use_fake = true
    fake_device_location = "."
    fake_device_type = "SimpleFake"
        "#;

        let test_config_file = "config/configuring_with_test_mode_should_success.toml";
        let mut file = std::fs::File::create(test_config_file).expect("Failed to create test file");
        file.write(test_config_str.as_bytes())
            .expect("Failed to write test config file");

        let config = get_config(&RunMode::Test(test_config_file.to_string()));
        let config = config.unwrap();

        assert_eq!(config.log.level, "debug");
        assert_eq!(config.devices.use_fake, true);
        assert_eq!(config.devices.fake_device_location, ".");
        assert_eq!(config.devices.fake_device_type, "SimpleFake");

        std::fs::remove_file(test_config_file).expect("Failed to remove test file");
    }
}
