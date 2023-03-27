use super::{device_info::DeviceInfo, BlockDevice, DataBlock, UNMAP_BLOCK};

pub struct SimpleFakeDevice {
    device_info: DeviceInfo,
    data: Vec<DataBlock>,
}

impl SimpleFakeDevice {
    pub fn new(name: String, size: u64) -> Result<Self, String> {
        let device_info = DeviceInfo::new(name, size)?;

        let mut data = Vec::new();
        for _ in 0..device_info.num_blocks() {
            data.push(UNMAP_BLOCK);
        }

        Ok(SimpleFakeDevice {
            device_info: device_info,
            data,
        })
    }

    fn is_valid_range(&self, lba: u64, num_blocks: u64) -> bool {
        if num_blocks == 0 || lba + num_blocks > self.device_info.num_blocks() {
            false
        } else {
            true
        }
    }
}

impl BlockDevice for SimpleFakeDevice {
    fn write(&mut self, lba: u64, num_blocks: u64, buffer: Vec<DataBlock>) -> Result<(), String> {
        if self.is_valid_range(lba, num_blocks) == false {
            return Err("Invalid lba ranges".to_string());
        }

        if (buffer.len() as u64) < num_blocks {
            return Err("Not enough buffers provided".to_string());
        }

        for offset in 0..num_blocks {
            let current_lba = (lba + offset) as usize;
            self.data[current_lba] = buffer[offset as usize];
        }

        Ok(())
    }

    fn read(&self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String> {
        if self.is_valid_range(lba, num_blocks) == false {
            return Err("Invalid lba ranges".to_string());
        }

        let mut read_data = Vec::new();
        for offset in 0..num_blocks {
            let current_lba = (lba + offset) as usize;
            read_data.push(self.data[current_lba].clone());
        }

        Ok(read_data)
    }

    fn info(&self) -> &DeviceInfo {
        &self.device_info
    }
}

#[cfg(test)]
mod tests {
    use super::super::BLOCK_SIZE;
    use super::*;

    #[test]
    fn create_block_device_with_unaligned_size_should_fail() {
        let device = SimpleFakeDevice::new(
            "block_device_should_provide_correct_device_info".to_string(),
            1000000,
        );

        assert_eq!(device.is_err(), true);
    }

    #[test]
    fn block_device_should_provide_correct_device_info() {
        let device = SimpleFakeDevice::new(
            "block_device_should_provide_correct_device_info".to_string(),
            BLOCK_SIZE as u64 * 1000,
        )
        .expect("Failed to create fake device");

        let info = device.info();
        assert_eq!(
            info.name(),
            &"block_device_should_provide_correct_device_info".to_string()
        );
        assert_eq!(info.device_size(), BLOCK_SIZE as u64 * 1000);
        assert_eq!(info.num_blocks(), 1000);
    }

    #[test]
    fn write_and_read_should_success() {
        let mut device = SimpleFakeDevice::new(
            "write_and_read_should_success".to_string(),
            BLOCK_SIZE as u64 * 1024,
        )
        .expect("Failed to create fake device");

        let lba = 10;
        let num_blocks = 5;
        let mut buffers = Vec::new();
        for num in 0..num_blocks {
            let block_buffer = [num as u8 as u8; BLOCK_SIZE];
            buffers.push(block_buffer);
        }
        assert_eq!(
            device
                .write(lba, num_blocks, buffers.clone().to_vec())
                .is_ok(),
            true
        );

        let read_result = device.read(lba, num_blocks);
        assert_eq!(read_result.is_ok(), true);
        assert_eq!(read_result.unwrap(), buffers);
    }

    #[test]
    fn write_with_invalid_lba_range_should_fail() {
        let mut device = SimpleFakeDevice::new(
            "write_with_invalid_lba_range_should_fail".to_string(),
            BLOCK_SIZE as u64 * 1024,
        )
        .expect("Failed to create fake device");

        let buffer = Vec::new();
        assert_eq!(device.write(0, 2000, buffer.clone()).is_err(), true);
        assert_eq!(device.write(0, 0, buffer.clone()).is_err(), true);
    }

    #[test]
    fn write_should_fail_when_not_enough_buffer_is_provided() {
        let mut device = SimpleFakeDevice::new(
            "write_should_fail_when_not_enough_buffer_is_provided".to_string(),
            BLOCK_SIZE as u64 * 1024,
        )
        .expect("Failed to create fake device");

        let mut buffer = Vec::new();
        for offset in 0..5 {
            buffer.push([offset as u8; BLOCK_SIZE]);
        }
        assert_eq!(device.write(0, 10, buffer.clone()).is_err(), true);
    }

    #[test]
    fn read_with_invalid_lba_range_should_fail() {
        let device = SimpleFakeDevice::new(
            "read_with_invalid_lba_range_should_fail".to_string(),
            BLOCK_SIZE as u64 * 1024,
        )
        .expect("Failed to create fake device");

        assert_eq!(device.read(0, 2000).is_err(), true);
        assert_eq!(device.read(0, 0).is_err(), true);
    }
}
