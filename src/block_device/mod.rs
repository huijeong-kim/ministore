use self::{data_type::DataBlock, device_info::DeviceInfo, simple_fake_device::SimpleFakeDevice};
use strum_macros::{Display, EnumIter};

mod data_type;
mod device_info;
pub mod io_uring_fake_device;
pub mod simple_fake_device;

pub trait BlockDevice {
    fn info(&self) -> &DeviceInfo;
    fn write(&mut self, lba: u64, num_blocks: u64, buffer: Vec<DataBlock>) -> Result<(), String>;
    fn read(&self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String>;
    fn load(&mut self) -> Result<(), String>;
    fn flush(&mut self) -> Result<(), String>;
}

#[derive(Debug, EnumIter, Clone, Display)]
pub enum BlockDeviceType {
    SimpleFakeDevice,
}

fn create_block_device(
    device_type: BlockDeviceType,
    name: String,
    size: u64,
) -> Result<Box<dyn BlockDevice>, String> {
    match device_type {
        BlockDeviceType::SimpleFakeDevice => {
            let fake = SimpleFakeDevice::new(name, size)?;
            Ok(Box::new(fake))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::data_type::*;
    use super::*;
    use strum::IntoEnumIterator;

    fn for_each_block_device_type<F>(mut f: F)
    where
        F: FnMut(BlockDeviceType) -> () + std::panic::UnwindSafe,
    {
        for device_type in BlockDeviceType::iter() {
            if let Err(e) = catch_assertion_failure(std::panic::AssertUnwindSafe(|| {
                f(device_type.clone());
            })) {
                panic!("{}, device_type={}", e, device_type);
            }
        }
    }

    fn catch_assertion_failure<F>(f: F) -> Result<(), String>
    where
        F: FnOnce() -> () + std::panic::UnwindSafe,
    {
        let result = std::panic::catch_unwind(|| {
            f();
        });

        if let Err(panic) = result {
            if let Some(message) = panic.downcast_ref::<&str>() {
                return Err(message.clone().into());
            }
        }
        Ok(())
    }

    #[test]
    fn create_block_device_with_unaligned_size_should_fail() {
        for_each_block_device_type(|device_type| {
            let device = create_block_device(
                device_type,
                "block_device_should_provide_correct_device_info".to_string(),
                1000000,
            );

            assert!(device.is_err() == true);
        });
    }

    #[test]
    fn block_device_should_provide_correct_device_info() {
        for_each_block_device_type(|device_type| {
            let device = create_block_device(
                device_type.clone(),
                "block_device_should_provide_correct_device_info".to_string(),
                BLOCK_SIZE as u64 * 1000,
            )
            .expect(&format!("Failed to create a device, type={}", device_type));

            let info = device.info();
            assert_eq!(
                info.name(),
                &"block_device_should_provide_correct_device_info".to_string()
            );
            assert_eq!(info.device_size(), BLOCK_SIZE as u64 * 1000);
            assert_eq!(info.num_blocks(), 1000);
        });
    }

    #[test]
    fn write_and_read_should_success() {
        for_each_block_device_type(|device_type| {
            let mut device = create_block_device(
                device_type,
                "write_and_read_should_success".to_string(),
                BLOCK_SIZE as u64 * 1024,
            )
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
                    .is_ok(),
                true
            );

            let read_result = device.read(lba, num_blocks);
            assert_eq!(read_result.is_ok(), true);
            assert_eq!(read_result.unwrap(), buffers);
        });
    }

    #[test]
    fn write_with_invalid_lba_range_should_fail() {
        for_each_block_device_type(|device_type| {
            let mut device = create_block_device(
                device_type,
                "write_with_invalid_lba_range_should_fail".to_string(),
                BLOCK_SIZE as u64 * 1024,
            )
            .expect("Failed to create fake device");

            let buffer = Vec::new();
            assert_eq!(device.write(0, 2000, buffer.clone()).is_err(), true);
            assert_eq!(device.write(0, 0, buffer.clone()).is_err(), true);
        });
    }

    #[test]
    fn write_should_fail_when_not_enough_buffer_is_provided() {
        for_each_block_device_type(|device_type| {
            let mut device = create_block_device(
                device_type,
                "write_should_fail_when_not_enough_buffer_is_provided".to_string(),
                BLOCK_SIZE as u64 * 1024,
            )
            .expect("Failed to create fake device");

            let mut buffer = Vec::new();
            for offset in 0..5 {
                buffer.push(DataBlock([offset as u8; BLOCK_SIZE]));
            }
            assert_eq!(device.write(0, 10, buffer.clone()).is_err(), true);
        });
    }

    #[test]
    fn read_with_invalid_lba_range_should_fail() {
        for_each_block_device_type(|device_type| {
            let device = create_block_device(
                device_type,
                "read_with_invalid_lba_range_should_fail".to_string(),
                BLOCK_SIZE as u64 * 1024,
            )
            .expect("Failed to create fake device");

            assert_eq!(device.read(0, 2000).is_err(), true);
            assert_eq!(device.read(0, 0).is_err(), true);
        });
    }
}
