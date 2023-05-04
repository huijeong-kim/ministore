use crate::config::{EnvironmentVariables, MinistoreConfig};
use crate::{device_manager::DeviceManager, grpc_server::GrpcServer};

pub mod async_block_device;
pub mod block_device;
pub mod block_device_common;
pub mod config;
pub mod device_manager;
pub mod grpc_server;
pub mod utils;

pub fn start(configs: (MinistoreConfig, EnvironmentVariables)) -> Result<(), String> {
    // Instantiate building blocks
    let device_manager = DeviceManager::new(&configs.0.devices)?;
    let grpc_server = GrpcServer::new(device_manager);
    let grpc_server_addr = format!("{}:{}", configs.1.server_addr, configs.1.server_port);

    // Run server
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to setup tokio runtime");

    runtime.block_on(async move {
        grpc_server::start_grpc_server(grpc_server_addr.as_str(), grpc_server)
            .await
            .expect("Failed to start gRPC server");
    });

    Ok(())
}

pub fn get_config(config_str: &str) -> Result<MinistoreConfig, String> {
    config::get_config(config_str)
}
