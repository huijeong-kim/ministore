use std::path::Path;

use crate::block_device_common::{
    data_type::{DataBlock, UNMAP_BLOCK},
    device_info::DeviceInfo,
    BlockDeviceType,
};
use tokio::{fs, io::AsyncWriteExt};

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone)]
pub struct Data(pub Vec<DataBlock>);
impl Data {
    pub fn new(size: usize) -> Self {
        let mut items = Vec::new();
        for _ in 0..size {
            items.push(UNMAP_BLOCK);
        }

        Self(items)
    }
}

#[derive(Serialize, Deserialize)]
pub struct AsyncSimpleFakeDevice {
    device_info: DeviceInfo,
    data: Data,
}
impl std::fmt::Debug for AsyncSimpleFakeDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncSimpleFakeDevice")
            .field("device_info", &self.device_info)
            .finish()
    }
}
impl AsyncSimpleFakeDevice {
    pub async fn new(
        device_type: BlockDeviceType,
        name: String,
        size: u64,
    ) -> Result<Self, String> {
        if device_type != BlockDeviceType::AsyncSimpleFakeDevice {
            return Err(format!(
                "Cannot create AsyncSimpleFakeDevice for type {}",
                device_type
            ));
        }

        let device_info = DeviceInfo::new(device_type, name, size)?;
        let num_blocks = device_info.num_blocks();
        let mut device = AsyncSimpleFakeDevice {
            device_info,
            data: Data::new(num_blocks as usize),
        };

        if fs::try_exists(device.info().name()).await.unwrap_or(false) == false {
            device.flush().await?;
        }

        Ok(device)
    }

    fn is_valid_range(&self, lba: u64, num_blocks: u64) -> bool {
        if num_blocks == 0 || lba + num_blocks > self.device_info.num_blocks() {
            false
        } else {
            true
        }
    }

    pub fn info(&self) -> &DeviceInfo {
        &self.device_info
    }

    pub async fn write(
        &mut self,
        lba: u64,
        num_blocks: u64,
        buffer: Vec<DataBlock>,
    ) -> Result<(), String> {
        if self.is_valid_range(lba, num_blocks) == false {
            return Err("Invalid lba ranges".to_string());
        }

        if (buffer.len() as u64) < num_blocks {
            return Err("Not enough buffers provided".to_string());
        }

        for offset in 0..num_blocks {
            let current_lba = (lba + offset) as usize;
            self.data.0[current_lba] = buffer[offset as usize];
        }

        Ok(())
    }

    pub async fn read(&mut self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String> {
        if self.is_valid_range(lba, num_blocks) == false {
            return Err("Invalid lba ranges".to_string());
        }

        let mut read_data = Vec::new();
        for offset in 0..num_blocks {
            let current_lba = (lba + offset) as usize;
            read_data.push(self.data.0[current_lba].clone());
        }

        Ok(read_data)
    }

    pub async fn load(&mut self) -> Result<(), String> {
        let filename = self.device_info.name().clone();
        let path = Path::new(&filename);

        if !path.exists() {
            return Err("No files to load".to_string());
        }

        let data = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
        let loaded_data: AsyncSimpleFakeDevice =
            bincode::deserialize_from(data.as_slice()).map_err(|e| e.to_string())?;

        if loaded_data.device_info.device_type() != BlockDeviceType::AsyncSimpleFakeDevice {
            return Err("Loaded device file type is invalid".to_string());
        }

        self.device_info = loaded_data.device_info;
        self.data = loaded_data.data;

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), String> {
        let filename = self.device_info.name().clone();
        let path = Path::new(&filename);

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .await
            .map_err(|e| e.to_string())?;

        let data = bincode::serialize(&self).map_err(|e| e.to_string())?;
        file.write_all(data.as_slice())
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
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

    #[tokio::test]
    async fn async_device_should_be_able_to_provide_device_info_after_load() {
        let device_name =
            "async_device_should_be_able_to_provide_device_info_after_load".to_string();

        {
            let mut device = AsyncSimpleFakeDevice::new(
                BlockDeviceType::AsyncSimpleFakeDevice,
                device_name.clone(),
                BLOCK_SIZE as u64 * 1024,
            )
            .await
            .expect("Failed to create block device");

            device.flush().await.expect("Failed to flush data");
        }

        {
            let mut device = AsyncSimpleFakeDevice::new(
                BlockDeviceType::AsyncSimpleFakeDevice,
                device_name.clone(),
                0,
            )
            .await
            .expect("Failed to load a device");

            device.load().await.expect("Failed to load data");

            assert_eq!(device.info().name(), &device_name);
            assert_eq!(device.info().device_size(), BLOCK_SIZE as u64 * 1024);
        }

        std::fs::remove_file(&device_name).expect("Failed to remove test file");
    }
}
