use std::sync::Once;

use ministore::{
    self,
    config::EnvironmentVariables,
    grpc_server::ministore_proto::{
        self, mini_service_client::MiniServiceClient, CreateFakeDeviceRequest,
        DeleteFakeDeviceRequest, ListFakeDevicesRequest, ReadRequest, WriteRequest,
    },
};

/// We will use 127.0.0.1:81** for gRPC server address of integration tests

static LOGGER_INITIALIZED: Once = Once::new();

#[test]
fn test_simple_io_flow_using_simple_fake_devices() {
    LOGGER_INITIALIZED.call_once(|| {
        ministore::telemetry::init_tracing("trace").expect("Failed to init tracing");
    });

    // Prepare configuration for test
    let ministore_config = r#"[devices]
    use_fake = true
    fake_device_location = "test_simple_io_flow_using_simple_fake_devices"
    fake_device_type = "SimpleFake"
        "#;

    let environment_variables = EnvironmentVariables {
        server_addr: "127.0.0.1".to_string(),
        server_port: "8100".to_string(),
        log_level: "trace".to_string(),
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    // Start ministore
    let server_handle = runtime.spawn(async {
        ministore::start((ministore_config, environment_variables))
            .await
            .unwrap();
    });

    // Start test here
    let test_handle = runtime.spawn(async {
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

    runtime.block_on(async {
        tokio::join!(test_handle).0.unwrap();
        server_handle.abort();
    });

    // cleanup teardown directory and config file
    std::fs::remove_dir("test_simple_io_flow_using_simple_fake_devices").unwrap();
}

#[test]
fn test_concurrent_writes() {
    LOGGER_INITIALIZED.call_once(|| {
        ministore::telemetry::init_tracing("trace").expect("Failed to init tracing");
    });

    // Prepare configuration for test
    let ministore_config = r#"[devices]
    use_fake = true
    fake_device_location = "test_concurrent_writes"
    fake_device_type = "SimpleFake"
        "#;

    let environment_variables = EnvironmentVariables {
        server_addr: "127.0.0.1".to_string(),
        server_port: "8101".to_string(),
        log_level: "trace".to_string(),
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    // Start ministore
    let server_handle = runtime.spawn(async {
        ministore::start((ministore_config, environment_variables))
            .await
            .unwrap();
    });

    // Start test here
    let test_handle = runtime.spawn(async {
        let addr = "http://127.0.0.1:8101";

        let mut client = loop {
            if let Ok(client) = MiniServiceClient::connect(addr.clone()).await {
                break client;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        };

        // 1. Create fake device
        let request = tonic::Request::new(CreateFakeDeviceRequest {
            name: "test_concurrent_writes".to_string(),
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
            "test_concurrent_writes"
        );

        // 2. Write some data
        let mut tasks = Vec::new();

        for concurrent_id in 0..100 {
            let handle = tokio::spawn(async move {
                for task in 0..10 {
                    let write_data = ministore_proto::Data {
                        data: vec![vec![task as u8; 4096]],
                    };
                    let request = tonic::Request::new(WriteRequest {
                        name: "test_concurrent_writes".to_string(),
                        lba: concurrent_id % 32,
                        num_blocks: 1,
                        data: Some(write_data.clone()),
                    });

                    let mut client = loop {
                        if let Ok(client) = MiniServiceClient::connect(addr.clone()).await {
                            break client;
                        } else {
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                    };

                    let response = client.write(request).await.unwrap();
                    let response = response.into_inner();

                    assert_eq!(response.success, true, "{:?}", response);
                }
            });

            tasks.push(handle);
        }

        for task in tasks {
            tokio::join!(task).0.expect("Failed to join tokio task");
        }

        // 3. Delete the device
        let request = tonic::Request::new(DeleteFakeDeviceRequest {
            name: "test_concurrent_writes".to_string(),
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

    runtime.block_on(async {
        tokio::join!(test_handle).0.unwrap();
        server_handle.abort();
    });

    // cleanup teardown directory and config file
    std::fs::remove_dir("test_concurrent_writes").unwrap();
}
