use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MinistoreConfig {
    pub devices: DeviceConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeviceConfig {
    pub use_fake: bool,
    pub fake_device_location: String,
    pub fake_device_type: String,
}

pub fn get_config(config_str: &str) -> Result<MinistoreConfig, String> {
    let config = Config::builder()
        .add_source(File::with_name("config/default.toml"))
        .add_source(File::from_str(config_str, config::FileFormat::Toml))
        .build()
        .map_err(|e| e.to_string())?;

    let config: MinistoreConfig = config.try_deserialize().map_err(|e| e.to_string())?;

    Ok(config)
}

#[derive(Debug)]
pub struct EnvironmentVariables {
    pub server_addr: String,
    pub server_port: String,
    pub log_level: String,
}
