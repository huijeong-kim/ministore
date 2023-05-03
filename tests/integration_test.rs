use std::{io::Write, path::PathBuf};

use dotenv::dotenv;
use ministore::{
    self,
    grpc_server::ministore_proto::{
        self, mini_service_client::MiniServiceClient, CreateFakeDeviceRequest,
        DeleteFakeDeviceRequest, ListFakeDevicesRequest, ReadRequest, WriteRequest,
    },
};

fn setup_config_file(config_file: &PathBuf, config_str: &str) {
    let mut file = std::fs::File::create(config_file).unwrap();
    file.write(config_str.as_bytes()).unwrap();
}
fn teardown_config_file(config_file: &PathBuf) {
    std::fs::remove_file(config_file).unwrap();
}

fn get_ministore_addr() -> (String, String) {
    dotenv().ok();
    let addr = std::env::var("MINISTORE_SERVER_ADDR").unwrap();
    let port = std::env::var("MINISTORE_SERVER_PORT").unwrap();

    (addr, port)
}

#[test]
fn test_simple_io_flow_using_simple_fake_devices() {
    // Prepare configuration for test
    let test_config_str = r#"[log]
    level = "debug"

    [devices]
    use_fake = true
    fake_device_location = "test_simple_io_flow_using_simple_fake_devices"
    fake_device_type = "SimpleFake"
        "#;
    let config_file = PathBuf::from("test_simple_io_flow_using_simple_fake_devices.toml");
    let config_file_as_string = config_file.clone().into_os_string().into_string().unwrap();
    setup_config_file(&config_file, test_config_str);

    // Start ministore
    let _thread = std::thread::spawn(|| {
        let run_mode: ministore::RunMode = ministore::RunMode::Test(config_file_as_string);
        ministore::start(run_mode).unwrap();
    });

    // Start test here
    let (addr, port) = get_ministore_addr();

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let addr = format!("http://{}:{}", addr, port);

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
    teardown_config_file(&config_file);
}
