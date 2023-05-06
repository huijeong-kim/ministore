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

pub fn get_config(config_str: &str) -> Result<MinistoreConfig, String> {
    todo!()
}

#[derive(Debug)]
pub struct EnvironmentVariables {
    pub server_addr: String,
    pub server_port: String,
}
