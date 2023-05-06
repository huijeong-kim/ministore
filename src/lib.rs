use crate::config::EnvironmentVariables;
use crate::{device_manager::DeviceManager, grpc_server::GrpcServer};

pub mod async_block_device;
pub mod block_device;
pub mod block_device_common;
pub mod config;
pub mod device_manager;
pub mod grpc_server;
pub mod utils;

pub fn start(configs: (&str, EnvironmentVariables)) -> Result<(), String> {
    todo!()

    // Instantiate building blocks

    // Run server

}
