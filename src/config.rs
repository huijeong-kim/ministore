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
    let config = Config::builder()
        .add_source(File::with_name("config/default.toml"))
        .add_source(File::with_name(&format!("config/{}.toml", run_mode)))
        .build()
        .map_err(|e| e.to_string())?;

    let config: MinistoreConfig = config.try_deserialize().map_err(|e| e.to_string())?;

    Ok(config)
}

#[cfg(test)]
mod tests {
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
}
