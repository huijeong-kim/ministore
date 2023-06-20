use crate::config::EnvironmentVariables;
use crate::{device_manager::DeviceManager, grpc_server::GrpcServer};

pub mod async_block_device;
pub mod block_device;
pub mod block_device_common;
pub mod config;
pub mod device_manager;
pub mod grpc_server;
pub mod telemetry;
pub mod utils;

pub async fn start(configs: (&str, EnvironmentVariables)) -> Result<(), String> {
    let config = config::get_config(configs.0)?;

    tracing::info!("Starting ministore...");
    tracing::info!("config: {:#?}", config);
    tracing::info!("environment variables: {:#?}", configs.1);

    // Instantiate building blocks
    todo!();

    // Run server
    todo!();

    Ok(())
}
