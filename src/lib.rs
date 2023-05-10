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
    let device_manager = DeviceManager::new(&config.devices)?;
    let grpc_server = GrpcServer::new(device_manager);
    let grpc_server_addr = format!("{}:{}", configs.1.server_addr, configs.1.server_port);

    // Run server
    grpc_server::start_grpc_server(grpc_server_addr.as_str(), grpc_server)
        .await
        .expect("Failed to start gRPC server");

    Ok(())
}
