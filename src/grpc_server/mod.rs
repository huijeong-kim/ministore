use std::sync::{Arc, Mutex};
use tonic::transport::Server;
use tonic::Response;

use uuid::Uuid;

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
    let addr = addr.parse().map_err(|e| format!("{:?}", e))?;

    Server::builder()
        .add_service(MiniServiceServer::new(grpc_server))
        .serve(addr)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub struct GrpcServer {
    device_manager: Arc<Mutex<DeviceManager>>,
}

impl GrpcServer {
    pub fn new(device_manager: DeviceManager) -> Self {
        Self {
            device_manager: Arc::new(Mutex::new(device_manager)),
        }
    }
}

fn data_block_into_proto_data(
    data: Vec<crate::block_device_common::data_type::DataBlock>,
) -> ministore_proto::Data {
    ministore_proto::Data {
        data: data.iter().map(|d| d.0.to_vec()).collect(),
    }
}

fn proto_data_into_data_block(
    data: ministore_proto::Data,
) -> Vec<crate::block_device_common::data_type::DataBlock> {
    // Each block data should be size of BLOCK_SIZE
    data.data
        .into_iter()
        .map(|d| crate::block_device_common::data_type::DataBlock(d.to_vec().try_into().unwrap()))
        .collect()
}

fn is_valid_proto_data(data: &Option<ministore_proto::Data>) -> bool {
    if data.is_none() {
        return false;
    } else {
        for d in &data.as_ref().unwrap().data {
            if d.len() % crate::block_device_common::data_type::BLOCK_SIZE != 0 {
                return false;
            }
        }
    }

    true
}

#[tonic::async_trait]
impl MiniService for GrpcServer {
    async fn status(
        &self,
        _request: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        let reply = StatusResponse {
            status: Status::Ready as i32,
        };

        Ok(Response::new(reply))
    }

    async fn read(
        &self,
        request: tonic::Request<ReadRequest>,
    ) -> Result<tonic::Response<ReadResponse>, tonic::Status> {
        let request = request.into_inner();
        let request_id = Uuid::new_v4();

        let request_span = tracing::trace_span!(
            "Read data",
            %request_id,
        );
        let _request_span_guard = request_span.enter();

        tracing::trace!(
            "request_id={}, device={}, lba={}, num_blocks={}",
            request_id,
            request.name,
            request.lba,
            request.num_blocks
        );

        let reply = match self.device_manager.lock().unwrap().read(
            &request.name,
            request.lba,
            request.num_blocks,
        ) {
            Ok(data) => ReadResponse {
                success: true,
                data: Some(data_block_into_proto_data(data)),
                reason: None,
            },
            Err(e) => {
                tracing::error!("{:?}", e);
                ReadResponse {
                    success: false,
                    data: None,
                    reason: Some(e),
                }
            }
        };

        Ok(Response::new(reply))
    }

    async fn write(
        &self,
        request: tonic::Request<WriteRequest>,
    ) -> Result<tonic::Response<WriteResponse>, tonic::Status> {
        let request = request.into_inner();
        let request_id = Uuid::new_v4();

        let request_span = tracing::trace_span!("Write data", %request_id);
        let _request_span_guard = request_span.enter();
        tracing::trace!(
            "request_id={}, device={}, lba={}, num_blocks={}",
            request_id,
            request.name,
            request.lba,
            request.num_blocks
        );

        let reply = if is_valid_proto_data(&request.data) == false {
            tracing::error!("No data provided");
            WriteResponse {
                success: false,
                reason: Some("No data provided".to_string()),
            }
        } else {
            match self.device_manager.lock().unwrap().write(
                &request.name,
                request.lba,
                request.num_blocks,
                proto_data_into_data_block(request.data.unwrap()),
            ) {
                Ok(()) => WriteResponse {
                    success: true,
                    reason: None,
                },
                Err(e) => {
                    tracing::error!("{:?}", e);
                    WriteResponse {
                        success: false,
                        reason: Some(e),
                    }
                }
            }
        };

        Ok(Response::new(reply))
    }

    async fn create_fake_device(
        &self,
        request: tonic::Request<CreateFakeDeviceRequest>,
    ) -> Result<tonic::Response<CreateFakeDeviceResponse>, tonic::Status> {
        let request = request.into_inner();
        let request_id = Uuid::new_v4();

        let request_span = tracing::debug_span!(
            "Create a new fake device",
            %request_id
        );
        let _request_span_guard = request_span.enter();
        tracing::debug!(
            "request_id={}, device={}, size={}",
            request_id,
            request.name,
            request.size
        );

        let reply = match self
            .device_manager
            .lock()
            .unwrap()
            .create_fake_device(&request.name, request.size)
        {
            Ok(()) => CreateFakeDeviceResponse {
                success: true,
                reason: None,
            },
            Err(e) => {
                tracing::error!("{:?}", e);
                CreateFakeDeviceResponse {
                    success: false,
                    reason: Some(e),
                }
            }
        };

        Ok(Response::new(reply))
    }

    async fn delete_fake_device(
        &self,
        request: tonic::Request<DeleteFakeDeviceRequest>,
    ) -> Result<tonic::Response<DeleteFakeDeviceResponse>, tonic::Status> {
        let request = request.into_inner();
        let request_id = Uuid::new_v4();

        let request_span = tracing::debug_span!(
            "Delete a fake device",
            %request_id,
        );
        let _request_span_guard = request_span.enter();
        tracing::debug!("request_id={}, device={}", request_id, request.name);

        let reply = match self
            .device_manager
            .lock()
            .unwrap()
            .delete_fake_device(&request.name)
        {
            Ok(()) => DeleteFakeDeviceResponse {
                success: true,
                reason: None,
            },
            Err(e) => {
                tracing::error!("{:?}", e);
                DeleteFakeDeviceResponse {
                    success: false,
                    reason: Some(e),
                }
            }
        };

        Ok(Response::new(reply))
    }

    async fn list_fake_devices(
        &self,
        request: tonic::Request<ListFakeDevicesRequest>,
    ) -> Result<tonic::Response<ListFakeDevicesResponse>, tonic::Status> {
        let _request = request.into_inner();
        let request_id = Uuid::new_v4();

        let request_span = tracing::debug_span!(
            "List fake devices",
            %request_id,
        );
        let _request_span_guard = request_span.enter();
        tracing::debug!("request_id={}", request_id);

        let reply = match self.device_manager.lock().unwrap().list_fake_devices() {
            Ok(list) => ListFakeDevicesResponse {
                success: true,
                reason: None,
                device_list: list
                    .iter()
                    .map(|dev| FakeDevice {
                        name: dev.0.clone(),
                        size: dev.1,
                    })
                    .collect(),
            },
            Err(e) => {
                tracing::error!("{:?}", e);
                ListFakeDevicesResponse {
                    success: false,
                    reason: Some(e),
                    device_list: Vec::new(),
                }
            }
        };

        Ok(Response::new(reply))
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

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
    #[traced_test]
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
            let request = tonic::Request::new(StatusRequest {});

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
    #[traced_test]
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
    #[traced_test]
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
    #[traced_test]
    async fn server_should_reply_with_error_when_invalid_data_provided_for_write() {
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
                name: "server_should_reply_with_error_when_invalid_data_provided_for_write"
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
                name: "server_should_reply_with_error_when_invalid_data_provided_for_write"
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
                name: "server_should_reply_with_error_when_invalid_data_provided_for_write"
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
                name: "server_should_reply_with_error_when_invalid_data_provided_for_write"
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
