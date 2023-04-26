use std::path::Path;

use crate::block_device_common::{
    data_type::{DataBlock, UNMAP_BLOCK},
    device_info::DeviceInfo,
    BlockDeviceType,
};
use tokio::{fs, io::AsyncWriteExt};

pub struct AsyncSimpleFakeDevice {
}

impl AsyncSimpleFakeDevice {
    pub async fn new(
        device_type: BlockDeviceType,
        name: String,
        size: u64,
    ) -> Result<Self, String> {
        todo!()
    }

    pub fn info(&self) -> &DeviceInfo {
        todo!()
    }

    pub async fn write(
        &mut self,
        lba: u64,
        num_blocks: u64,
        buffer: Vec<DataBlock>,
    ) -> Result<(), String> {
        todo!()
    }

    pub async fn read(&mut self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String> {
        todo!()
    }

    pub async fn load(&mut self) -> Result<(), String> {
        todo!()
    }

    pub async fn flush(&mut self) -> Result<(), String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        async_block_device::async_simple_fake_device::AsyncSimpleFakeDevice,
        block_device_common::{data_type::*, BlockDeviceType},
    };
    use std::path::Path;

    #[tokio::test]
    async fn create_async_block_device_with_unaligned_size_should_fail() {
        let device = AsyncSimpleFakeDevice::new(
            BlockDeviceType::AsyncSimpleFakeDevice,
            "create_async_block_device_with_unaligned_size_should_fail".to_string(),
            1000000,
        )
        .await;

        assert!(device.is_err() == true);
    }

    #[tokio::test]
    async fn create_async_block_device_with_wrong_type_should_fail() {
        let device = AsyncSimpleFakeDevice::new(
            BlockDeviceType::SimpleFakeDevice,
            "create_async_block_device_with_wrong_type_should_fail".to_string(),
            1000000,
        )
        .await;

        assert_eq!(device.is_err(), true);
    }

    #[tokio::test]
    async fn async_block_device_should_create_file() {
        let device_name = "async_block_device_should_create_file".to_string();
        let _device = AsyncSimpleFakeDevice::new(
            BlockDeviceType::AsyncSimpleFakeDevice,
            device_name.clone(),
            BLOCK_SIZE as u64 * 1000,
        )
        .await
        .expect("Failed to create a device, type={}");

        assert_eq!(Path::new(&device_name).exists(), true);
        tokio::fs::remove_file(device_name)
            .await
            .expect("Failed to remove file");
    }

    #[tokio::test]
    async fn async_block_device_should_provide_correct_device_info() {
        let device_name = "async_block_device_should_provide_correct_device_info".to_string();
        let device = AsyncSimpleFakeDevice::new(
            BlockDeviceType::AsyncSimpleFakeDevice,
            device_name.clone(),
            BLOCK_SIZE as u64 * 1000,
        )
        .await
        .expect("Failed to create a device");

        let info = device.info();
        assert_eq!(info.name(), &device_name);
        assert_eq!(info.device_size(), BLOCK_SIZE as u64 * 1000);
        assert_eq!(info.num_blocks(), 1000);
        assert_eq!(info.device_type(), BlockDeviceType::AsyncSimpleFakeDevice);

        tokio::fs::remove_file(device_name)
            .await
            .expect("Failed to remove file");
    }

    #[tokio::test]
    async fn write_and_read_async_should_success() {
        let device_name = "write_and_read_async_should_success".to_string();
        let mut device = AsyncSimpleFakeDevice::new(
            BlockDeviceType::AsyncSimpleFakeDevice,
            device_name.clone(),
            BLOCK_SIZE as u64 * 1024,
        )
        .await
        .expect("Failed to create fake device");

        let lba = 10;
        let num_blocks = 5;
        let mut buffers = Vec::new();
        for num in 0..num_blocks {
            let block_buffer = DataBlock([num as u8 as u8; BLOCK_SIZE]);
            buffers.push(block_buffer);
        }
        assert_eq!(
            device
                .write(lba, num_blocks, buffers.clone().to_vec())
                .await
                .is_ok(),
            true
        );

        let read_result = device.read(lba, num_blocks).await;
        assert_eq!(read_result.is_ok(), true);
        assert_eq!(read_result.unwrap(), buffers);

        tokio::fs::remove_file(device_name)
            .await
            .expect("Failed to remove file");
    }

    #[tokio::test]
    async fn write_with_invalid_lba_range_async_should_fail() {
        let device_name = "write_with_invalid_lba_range_async_should_fail".to_string();
        let mut device = AsyncSimpleFakeDevice::new(
            BlockDeviceType::AsyncSimpleFakeDevice,
            device_name.clone(),
            BLOCK_SIZE as u64 * 1024,
        )
        .await
        .expect("Failed to create fake device");

        let buffer = Vec::new();
        assert_eq!(device.write(0, 2000, buffer.clone()).await.is_err(), true);
        assert_eq!(device.write(0, 0, buffer.clone()).await.is_err(), true);

        tokio::fs::remove_file(device_name)
            .await
            .expect("Failed to remove file");
    }

    #[tokio::test]
    async fn write_async_should_fail_when_not_enough_buffer_is_provided() {
        let device_name = "write_async_should_fail_when_not_enough_buffer_is_provided".to_string();
        let mut device = AsyncSimpleFakeDevice::new(
            BlockDeviceType::AsyncSimpleFakeDevice,
            device_name.clone(),
            BLOCK_SIZE as u64 * 1024,
        )
        .await
        .expect("Failed to create fake device");

        let mut buffer = Vec::new();
        for offset in 0..5 {
            buffer.push(DataBlock([offset as u8; BLOCK_SIZE]));
        }
        assert_eq!(device.write(0, 10, buffer.clone()).await.is_err(), true);

        tokio::fs::remove_file(device_name)
            .await
            .expect("Failed to remove file");
    }

    #[tokio::test]
    async fn read_async_with_invalid_lba_range_should_fail() {
        let device_name = "read_async_with_invalid_lba_range_should_fail".to_string();
        let mut device = AsyncSimpleFakeDevice::new(
            BlockDeviceType::AsyncSimpleFakeDevice,
            device_name.clone(),
            BLOCK_SIZE as u64 * 1024,
        )
        .await
        .expect("Failed to create fake device");

        assert_eq!(device.read(0, 2000).await.is_err(), true);
        assert_eq!(device.read(0, 0).await.is_err(), true);

        tokio::fs::remove_file(device_name)
            .await
            .expect("Failed to remove file");
    }

    #[tokio::test]
    async fn reading_unwritten_lbas_async_should_return_unmap_data() {
        let device_name = "reading_unwritten_lbas_async_should_return_unmap_data".to_string();
        let mut device = AsyncSimpleFakeDevice::new(
            BlockDeviceType::AsyncSimpleFakeDevice,
            device_name.clone(),
            BLOCK_SIZE as u64 * 1024,
        )
        .await
        .expect("Failed to create fake device");

        let read_data = device.read(0, 1).await.expect("Failed to read data");
        assert_eq!(read_data.len(), 1);
        assert_eq!(*read_data.get(0).unwrap(), UNMAP_BLOCK);

        tokio::fs::remove_file(device_name)
            .await
            .expect("Failed to remove file");
    }

    #[tokio::test]
    async fn flush_and_load_async_should_success() {
        let device_name = "flush_and_load_async_should_success".to_string();

        // Write data and flush all
        {
            let mut device = AsyncSimpleFakeDevice::new(
                BlockDeviceType::AsyncSimpleFakeDevice,
                device_name.clone(),
                BLOCK_SIZE as u64 * 1024,
            )
            .await
            .expect("Failed to create block device");

            device
                .write(
                    0,
                    3,
                    vec![
                        DataBlock([0xA; BLOCK_SIZE]),
                        DataBlock([0xB; BLOCK_SIZE]),
                        DataBlock([0xC; BLOCK_SIZE]),
                    ],
                )
                .await
                .expect("Failed to write data");

            device.flush().await.expect("Failed to flush data");
        }

        // Load data and verify
        {
            let mut device = AsyncSimpleFakeDevice::new(
                BlockDeviceType::AsyncSimpleFakeDevice,
                device_name.clone(),
                BLOCK_SIZE as u64 * 1024,
            )
            .await
            .expect("Failed to create block device");

            device.load().await.expect("Failed to load data");

            let read_data = device.read(0, 3).await.expect("Failed to read data");
            assert_eq!(
                read_data,
                vec![
                    DataBlock([0xA; BLOCK_SIZE]),
                    DataBlock([0xB; BLOCK_SIZE]),
                    DataBlock([0xC; BLOCK_SIZE])
                ]
            );
        }

        tokio::fs::remove_file(device_name)
            .await
            .expect("Failed to remove file");
    }
}
