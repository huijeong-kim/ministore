use ministore::{
    self,
    config::{DeviceConfig, EnvironmentVariables, LogConfig, MinistoreConfig},
    grpc_server::ministore_proto::{
        self, mini_service_client::MiniServiceClient, CreateFakeDeviceRequest,
        DeleteFakeDeviceRequest, ListFakeDevicesRequest, ReadRequest, WriteRequest,
    },
};

/// We will use 127.0.0.1:81** for gRPC server address of integration tests

#[test]
fn test_simple_io_flow_using_simple_fake_devices() {
    // Prepare configuration for test
    let ministore_config = MinistoreConfig {
        log: LogConfig {
            level: "debug".to_string(),
        },
        devices: DeviceConfig {
            use_fake: true,
            fake_device_location: "test_simple_io_flow_using_simple_fake_devices".to_string(),
            fake_device_type: "SimpleFake".to_string(),
        },
    };
    let environment_variables = EnvironmentVariables {
        server_addr: "127.0.0.1".to_string(),
        server_port: "8100".to_string(),
    };

    // Start ministore
    let _thread = std::thread::spawn(move || {
        ministore::start((ministore_config, environment_variables)).unwrap();
    });

    // Start test here
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let addr = "http://127.0.0.1:8100";

        let mut client = loop {
            if let Ok(client) = MiniServiceClient::connect(addr.clone()).await {
                break client;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        };

        // 1. Create fake device
        let request = tonic::Request::new(CreateFakeDeviceRequest {
            name: "test_simple_io_flow_using_simple_fake_devices".to_string(),
            size: 4 * 1024 * 32,
        });
        let response = client.create_fake_device(request).await.unwrap();
        let response = response.into_inner();

        assert_eq!(response.success, true, "{:?}", response);

        // and check if it's created
        let request = tonic::Request::new(ListFakeDevicesRequest {});
        let response = client.list_fake_devices(request).await.unwrap();
        let response = response.into_inner();

        assert_eq!(response.success, true, "{:?}", response);
        assert_eq!(response.device_list.len(), 1);
        assert_eq!(
            response.device_list.get(0).unwrap().name,
            "test_simple_io_flow_using_simple_fake_devices"
        );

        // 2. Write some data
        let write_data = ministore_proto::Data {
            data: vec![vec![7 as u8; 4096], vec![8 as u8; 4096]],
        };
        let request = tonic::Request::new(WriteRequest {
            name: "test_simple_io_flow_using_simple_fake_devices".to_string(),
            lba: 10,
            num_blocks: 2,
            data: Some(write_data.clone()),
        });
        let response = client.write(request).await.unwrap();
        let response = response.into_inner();

        assert_eq!(response.success, true, "{:?}", response);

        // 3. Read the data
        let request = tonic::Request::new(ReadRequest {
            name: "test_simple_io_flow_using_simple_fake_devices".to_string(),
            lba: 10,
            num_blocks: 2,
        });
        let response = client.read(request).await.unwrap();
        let response = response.into_inner();

        assert_eq!(response.success, true, "{:?}", response);
        assert_eq!(response.data.unwrap(), write_data);

        // 4. Delete the device
        let request = tonic::Request::new(DeleteFakeDeviceRequest {
            name: "test_simple_io_flow_using_simple_fake_devices".to_string(),
        });
        let response = client.delete_fake_device(request).await.unwrap();
        let response = response.into_inner();

        assert_eq!(response.success, true, "{:?}", response);

        // and check if it's deleted
        let request = tonic::Request::new(ListFakeDevicesRequest {});
        let response = client.list_fake_devices(request).await.unwrap();
        let response = response.into_inner();

        assert_eq!(response.success, true, "{:?}", response);
        assert_eq!(response.device_list.len(), 0);
    });

    // cleanup teardown directory and config file
    std::fs::remove_dir("test_simple_io_flow_using_simple_fake_devices").unwrap();
}
