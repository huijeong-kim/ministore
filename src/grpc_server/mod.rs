use std::sync::{Arc, Mutex};
use tonic::transport::Server;
use tonic::Response;

use crate::device_manager::DeviceManager;

use self::ministore_proto::mini_service_server::{MiniService, MiniServiceServer};
use self::ministore_proto::{
    CreateFakeDeviceRequest, CreateFakeDeviceResponse, DeleteFakeDeviceRequest,
    DeleteFakeDeviceResponse, FakeDevice, ListFakeDevicesRequest, ListFakeDevicesResponse,
    ReadRequest, ReadResponse, Status, StatusRequest, StatusResponse, WriteRequest, WriteResponse,
};

pub mod ministore_proto {
    tonic::include_proto!("ministore");
}

pub async fn start_grpc_server(addr: &str, grpc_server: GrpcServer) -> Result<(), String> {
    todo!()
}

pub struct GrpcServer {}

impl GrpcServer {
    pub fn new(device_manager: DeviceManager) -> Self {
        todo!()
    }
}

#[tonic::async_trait]
impl MiniService for GrpcServer {
    async fn status(
        &self,
        request: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    async fn read(
        &self,
        request: tonic::Request<ReadRequest>,
    ) -> Result<tonic::Response<ReadResponse>, tonic::Status> {
        todo!()
    }

    async fn write(
        &self,
        request: tonic::Request<WriteRequest>,
    ) -> Result<tonic::Response<WriteResponse>, tonic::Status> {
        todo!()
    }

    async fn create_fake_device(
        &self,
        request: tonic::Request<CreateFakeDeviceRequest>,
    ) -> Result<tonic::Response<CreateFakeDeviceResponse>, tonic::Status> {
        todo!()
    }

    async fn delete_fake_device(
        &self,
        request: tonic::Request<DeleteFakeDeviceRequest>,
    ) -> Result<tonic::Response<DeleteFakeDeviceResponse>, tonic::Status> {
        todo!()
    }

    async fn list_fake_devices(
        &self,
        request: tonic::Request<ListFakeDevicesRequest>,
    ) -> Result<tonic::Response<ListFakeDevicesResponse>, tonic::Status> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block_device_common::data_type::BLOCK_SIZE, config::DeviceConfig,
        grpc_server::ministore_proto::mini_service_client::MiniServiceClient,
        utils::humansize_to_integer,
    };

    use super::*;

    fn test_device_manager() -> DeviceManager {
        let config = DeviceConfig {
            use_fake: true,
            fake_device_location: "fakes".to_string(),
            fake_device_type: "SimpleFake".to_string(),
        };
        DeviceManager::new(&config).expect("Failed to create device manager")
    }

    /// Be sure to use different port for each test, so that all tests can be executed in parallel.
    #[tokio::test]
    async fn server_should_response_with_ready_when_started() {
        let addr = "127.0.0.1:8080";
        let addr_for_client = format!("http://{}", addr.clone());

        let start_server = tokio::spawn(async move {
            let grpc_server = GrpcServer::new(test_device_manager());
            start_grpc_server(addr, grpc_server)
                .await
                .expect("Failed to start grpc server");
        });

        let test = tokio::spawn(async move {
            let mut client = MiniServiceClient::connect(addr_for_client)
                .await
                .expect("Failed to start test client");
            let request: tonic::Request<StatusRequest> = tonic::Request::new(StatusRequest {});

            let response = client
                .status(request)
                .await
                .expect("Failed to get response");
            assert_eq!(response.into_inner().status, Status::Ready as i32);
        });

        test.await.unwrap();
        start_server.abort();
    }

    #[tokio::test]
    async fn server_should_be_able_to_create_and_delete_fake_device() {
        let addr = "127.0.0.1:8081";
        let addr_for_client = format!("http://{}", addr.clone());

        let start_server = tokio::spawn(async move {
            let grpc_server = GrpcServer::new(test_device_manager());
            start_grpc_server(addr, grpc_server)
                .await
                .expect("Failed to start grpc server");
        });

        let test = tokio::spawn(async move {
            let mut client = MiniServiceClient::connect(addr_for_client)
                .await
                .expect("Failed to start test client");

            // Create device and verify it using list devices
            let device_name = "server_can_create_and_delete_fake_device".to_string();
            let request = tonic::Request::new(CreateFakeDeviceRequest {
                name: device_name.clone(),
                size: humansize_to_integer("1M").unwrap(),
            });
            let response = client
                .create_fake_device(request)
                .await
                .expect("Failed to create fake device");
            let response = response.into_inner();
            assert_eq!(response.success, true, "{:?}", response);

            let request = tonic::Request::new(ListFakeDevicesRequest {});
            let response = client
                .list_fake_devices(request)
                .await
                .expect("Failed to get response");
            let response = response.into_inner();

            assert_eq!(response.success, true, "{:?}", response);
            assert_eq!(response.device_list.len(), 1);
            assert_eq!(response.device_list.get(0).unwrap().name, device_name);
            assert_eq!(
                response.device_list.get(0).unwrap().size,
                humansize_to_integer("1M").unwrap()
            );

            // Delete device and verify it using list devices
            let request = tonic::Request::new(DeleteFakeDeviceRequest {
                name: device_name.clone(),
            });
            let response = client
                .delete_fake_device(request)
                .await
                .expect("Failed to delete fake device");
            let response = response.into_inner();
            assert_eq!(response.success, true, "{:?}", response);

            let request = tonic::Request::new(ListFakeDevicesRequest {});
            let response = client
                .list_fake_devices(request)
                .await
                .expect("Failed to get response");
            let response = response.into_inner();

            assert_eq!(response.success, true, "{:?}", response);
            assert_eq!(response.device_list.len(), 0);
        });

        test.await.unwrap();
        start_server.abort();
    }

    #[tokio::test]
    async fn server_should_be_able_to_read_write_fake_device() {
        let addr = "127.0.0.1:8082";
        let addr_for_client = format!("http://{}", addr.clone());

        let start_server = tokio::spawn(async move {
            let grpc_server = GrpcServer::new(test_device_manager());
            start_grpc_server(addr, grpc_server)
                .await
                .expect("Failed to start grpc server");
        });

        let test = tokio::spawn(async move {
            let mut client = MiniServiceClient::connect(addr_for_client)
                .await
                .expect("Failed to start test client");

            // Create device for test
            let device_name = "server_should_be_able_to_read_write_fake_device".to_string();
            let request = tonic::Request::new(CreateFakeDeviceRequest {
                name: device_name.clone(),
                size: humansize_to_integer("1M").unwrap(),
            });
            let response = client
                .create_fake_device(request)
                .await
                .expect("Failed to create fake device");
            let response = response.into_inner();
            assert_eq!(response.success, true, "{:?}", response);

            // Write data to the device
            let write_data = ministore_proto::Data {
                data: vec![
                    vec![0xA as u8; BLOCK_SIZE],
                    vec![0xB as u8; BLOCK_SIZE],
                    vec![0xC as u8; BLOCK_SIZE],
                    vec![0xD as u8; BLOCK_SIZE],
                ],
            };
            let request = tonic::Request::new(WriteRequest {
                name: "server_should_be_able_to_read_write_fake_device".to_string(),
                lba: 10,
                num_blocks: 4,
                data: Some(write_data.clone()),
            });
            let response = client.write(request).await.expect("Failed to write data");
            let response = response.into_inner();
            assert_eq!(response.success, true, "{:?}", response);

            // Read data from the device
            let request = tonic::Request::new(ReadRequest {
                name: "server_should_be_able_to_read_write_fake_device".to_string(),
                lba: 10,
                num_blocks: 4,
            });
            let response = client.read(request).await.expect("Failed to read data");
            let response = response.into_inner();
            assert_eq!(response.success, true, "{:?}", response);

            // Verify read data
            assert_eq!(response.data.unwrap(), write_data);

            // Delete device for wrapup
            let request = tonic::Request::new(DeleteFakeDeviceRequest {
                name: "server_should_be_able_to_read_write_fake_device".to_string(),
            });
            let response = client
                .delete_fake_device(request)
                .await
                .expect("Failed to delete device");
            let response = response.into_inner();

            assert_eq!(response.success, true, "{:?}", response);
        });

        test.await.unwrap();
        start_server.abort();
    }

    #[tokio::test]
    async fn server_should_replay_with_error_when_invalid_data_provided_for_write() {
        let addr = "127.0.0.1:8083";
        let addr_for_client = format!("http://{}", addr.clone());

        let start_server = tokio::spawn(async move {
            let grpc_server = GrpcServer::new(test_device_manager());
            start_grpc_server(addr, grpc_server)
                .await
                .expect("Failed to start grpc server");
        });

        let test = tokio::spawn(async move {
            let mut client = MiniServiceClient::connect(addr_for_client)
                .await
                .expect("Failed to start test client");

            // Create device for test
            let request = tonic::Request::new(CreateFakeDeviceRequest {
                name: "server_should_replay_with_error_when_invalid_data_provided_for_write"
                    .to_string(),
                size: humansize_to_integer("1M").unwrap(),
            });
            let response = client
                .create_fake_device(request)
                .await
                .expect("Failed to create fake device");
            let response = response.into_inner();
            assert_eq!(response.success, true, "{:?}", response);

            // test 1. write request without data
            let invalid_request = tonic::Request::new(WriteRequest {
                name: "server_should_replay_with_error_when_invalid_data_provided_for_write"
                    .to_string(),
                lba: 0,
                num_blocks: 1,
                data: None,
            });
            let response = client
                .write(invalid_request)
                .await
                .expect("Failed to request write");
            let response = response.into_inner();
            assert_eq!(response.success, false);

            // test 2. write request with too-small data (smaller than the block size)
            let invalid_write_data = ministore_proto::Data {
                data: vec![vec![0xA as u8; 1024]],
            };
            let invalid_request = tonic::Request::new(WriteRequest {
                name: "server_should_replay_with_error_when_invalid_data_provided_for_write"
                    .to_string(),
                lba: 0,
                num_blocks: 1,
                data: Some(invalid_write_data),
            });
            let response = client
                .write(invalid_request)
                .await
                .expect("Failed to request write");
            let response = response.into_inner();
            assert_eq!(response.success, false);

            // Delete device for wrapup
            let request = tonic::Request::new(DeleteFakeDeviceRequest {
                name: "server_should_replay_with_error_when_invalid_data_provided_for_write"
                    .to_string(),
            });
            let response = client
                .delete_fake_device(request)
                .await
                .expect("Failed to delete device");
            let response = response.into_inner();
            assert_eq!(response.success, true, "{:?}", response);
        });

        test.await.unwrap();
        start_server.abort();
    }
}
