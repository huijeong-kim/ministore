use std::path::PathBuf;

use crate::block_device_common::data_type::DataBlock;
use crate::block_device_common::device_info::DeviceInfo;
use crate::block_device_common::BlockDeviceType;

use simple_fake_device::SimpleFakeDevice;

pub mod io_uring_fake_device;
pub mod simple_fake_device;

pub trait BlockDevice: Send + Sync {
    fn info(&self) -> &DeviceInfo;
    fn write(&mut self, lba: u64, num_blocks: u64, buffer: Vec<DataBlock>) -> Result<(), String>;
    fn read(&mut self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String>;
    fn load(&mut self) -> Result<(), String>;
    fn flush(&mut self) -> Result<(), String>;
}

pub fn create_block_device(
    device_type: BlockDeviceType,
    name: String,
    size: u64,
    filepath: PathBuf,
) -> Result<Box<dyn BlockDevice>, String> {
    match device_type {
        BlockDeviceType::SimpleFakeDevice => {
            let fake = SimpleFakeDevice::new(name, size, filepath)?;
            Ok(Box::new(fake))
        }
        BlockDeviceType::AsyncSimpleFakeDevice => {
            Err("Cannot create BlockDevice trait for AsyncSimpleFakeDevice".to_string())
        }
    }
}

#[cfg(target_os = "linux")]
fn create_io_uring_fake_device(
    name: String,
    size: u64,
    _filepath: PathBuf,
) -> Result<Box<dyn BlockDevice>, String> {
    let device = io_uring_fake_device::IoUringFakeDevice::new(name, size)?;
    Ok(Box::new(device))
}
#[cfg(not(target_os = "linux"))]
fn create_io_uring_fake_device(
    _name: String,
    _size: u64,
    _filepath: PathBuf,
) -> Result<Box<dyn BlockDevice>, String> {
    Err("Cannot create io uring fake device".to_string())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::block_device_common::data_type::*;
    use strum::IntoEnumIterator;

    #[cfg(target_os = "linux")]
    fn translate_device_type(device_type: BlockDeviceType) -> BlockDeviceType {
        device_type
    }

    #[cfg(not(target_os = "linux"))]
    fn translate_device_type(device_type: BlockDeviceType) -> BlockDeviceType {
        // Use SimpleFakeDevice instead when target os is not a linux
        match device_type {
            BlockDeviceType::SimpleFakeDevice => BlockDeviceType::SimpleFakeDevice,
            BlockDeviceType::AsyncSimpleFakeDevice => panic!("async type cannot be used here"),
        }
    }

    fn for_each_block_device_type<F>(mut f: F)
    where
        F: FnMut(BlockDeviceType) -> () + std::panic::UnwindSafe,
    {
        for device_type in BlockDeviceType::iter() {
            if device_type.is_sync() {
                let device_type = translate_device_type(device_type);
                if let Err(e) = catch_assertion_failure(std::panic::AssertUnwindSafe(|| {
                    f(device_type.clone());
                })) {
                    panic!("{}, device_type={}", e, device_type);
                }
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
            let message = match panic.downcast_ref::<String>() {
                Some(m) => m,
                None => "",
            };

            return Err(message.clone().into());
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
                PathBuf::from("."),
            );

            assert!(device.is_err() == true);
        });
    }

    #[test]
    fn block_device_should_create_file() {
        for_each_block_device_type(|device_type| {
            let device_name = "block_device_should_create_file".to_string();
            let _device = create_block_device(
                device_type.clone(),
                device_name.clone(),
                BLOCK_SIZE as u64 * 1000,
                PathBuf::from("."),
            )
            .expect(&format!("Failed to create a device, type={}", device_type));

            assert_eq!(Path::new(&device_name).exists(), true);
            std::fs::remove_file(device_name).expect("Failed to remove file");
        })
    }

    #[test]
    fn block_device_should_provide_correct_device_info() {
        for_each_block_device_type(|device_type| {
            let device_name = "block_device_should_provide_correct_device_info".to_string();
            let device = create_block_device(
                device_type.clone(),
                device_name.clone(),
                BLOCK_SIZE as u64 * 1000,
                PathBuf::from("."),
            )
            .expect(&format!("Failed to create a device, type={}", device_type));

            let info = device.info();
            assert_eq!(info.name(), &device_name);
            assert_eq!(info.device_size(), BLOCK_SIZE as u64 * 1000);
            assert_eq!(info.num_blocks(), 1000);
            assert_eq!(info.device_type(), device_type);

            std::fs::remove_file(&device_name).expect("Failed to remove file");
        });
    }

    #[test]
    fn write_and_read_should_success() {
        for_each_block_device_type(|device_type| {
            let device_name = "write_and_read_should_success".to_string();
            let mut device = create_block_device(
                device_type,
                device_name.clone(),
                BLOCK_SIZE as u64 * 1024,
                PathBuf::from("."),
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

            std::fs::remove_file(device_name).expect("Failed to remove file");
        });
    }

    #[test]
    fn write_with_invalid_lba_range_should_fail() {
        for_each_block_device_type(|device_type| {
            let device_name = "write_with_invalid_lba_range_should_fail".to_string();
            let mut device = create_block_device(
                device_type,
                device_name.clone(),
                BLOCK_SIZE as u64 * 1024,
                PathBuf::from("."),
            )
            .expect("Failed to create fake device");

            let buffer = Vec::new();
            assert_eq!(device.write(0, 2000, buffer.clone()).is_err(), true);
            assert_eq!(device.write(0, 0, buffer.clone()).is_err(), true);

            std::fs::remove_file(device_name).expect("Failed to remove file");
        });
    }

    #[test]
    fn write_should_fail_when_not_enough_buffer_is_provided() {
        for_each_block_device_type(|device_type| {
            let device_name = "write_should_fail_when_not_enough_buffer_is_provided".to_string();
            let mut device = create_block_device(
                device_type,
                device_name.clone(),
                BLOCK_SIZE as u64 * 1024,
                PathBuf::from("."),
            )
            .expect("Failed to create fake device");

            let mut buffer = Vec::new();
            for offset in 0..5 {
                buffer.push(DataBlock([offset as u8; BLOCK_SIZE]));
            }
            assert_eq!(device.write(0, 10, buffer.clone()).is_err(), true);

            std::fs::remove_file(device_name).expect("Failed to remove file");
        });
    }

    #[test]
    fn read_with_invalid_lba_range_should_fail() {
        for_each_block_device_type(|device_type| {
            let device_name = "read_with_invalid_lba_range_should_fail".to_string();
            let mut device = create_block_device(
                device_type,
                device_name.clone(),
                BLOCK_SIZE as u64 * 1024,
                PathBuf::from("."),
            )
            .expect("Failed to create fake device");

            assert_eq!(device.read(0, 2000).is_err(), true);
            assert_eq!(device.read(0, 0).is_err(), true);

            std::fs::remove_file(device_name).expect("Failed to remove file");
        });
    }

    #[test]
    fn reading_unwritten_lbas_should_return_unmap_data() {
        for_each_block_device_type(|device_type| {
            let device_name = "reading_unwritten_lbas_should_return_unmap_data".to_string();
            let mut device = create_block_device(
                device_type,
                device_name.clone(),
                BLOCK_SIZE as u64 * 1024,
                PathBuf::from("."),
            )
            .expect("Failed to create fake device");

            let read_data = device.read(0, 1).expect("Failed to read data");
            assert_eq!(read_data.len(), 1);
            assert_eq!(*read_data.get(0).unwrap(), UNMAP_BLOCK);

            std::fs::remove_file(device_name).expect("Failed to remove file");
        });
    }

    #[test]
    fn flush_and_load_should_success() {
        for_each_block_device_type(|device_type| {
            let device_name = "flush_and_load_should_success".to_string();

            // Write data and flush all
            {
                let mut device = create_block_device(
                    device_type.clone(),
                    device_name.clone(),
                    BLOCK_SIZE as u64 * 1024,
                    PathBuf::from("."),
                )
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
                    .expect("Failed to write data");

                device.flush().expect("Failed to flush data");
            }

            // Load data and verify
            {
                let mut device = create_block_device(
                    device_type,
                    device_name.clone(),
                    BLOCK_SIZE as u64 * 1024,
                    PathBuf::from("."),
                )
                .expect("Failed to create block device");

                device.load().expect("Failed to load data");

                let read_data = device.read(0, 3).expect("Failed to read data");
                assert_eq!(
                    read_data,
                    vec![
                        DataBlock([0xA; BLOCK_SIZE]),
                        DataBlock([0xB; BLOCK_SIZE]),
                        DataBlock([0xC; BLOCK_SIZE])
                    ]
                );
            }

            std::fs::remove_file(&device_name).expect("Failed to remove test file");
        });
    }

    #[test]
    fn device_should_be_able_to_provide_device_info_after_load() {
        for_each_block_device_type(|device_type| {
            let device_name = "device_should_be_able_to_provide_device_info_after_load".to_string();

            {
                let mut device = create_block_device(
                    device_type.clone(),
                    device_name.clone(),
                    BLOCK_SIZE as u64 * 1024,
                    PathBuf::from("."),
                )
                .expect("Failed to create block device");

                device.flush().expect("Failed to flush data");
            }

            {
                let mut device = create_block_device(
                    device_type.clone(),
                    device_name.clone(),
                    0,
                    PathBuf::from("."),
                )
                .expect("Failed to load a device");
                device.load().expect("Failed to load data");

                assert_eq!(device.info().name(), &device_name);
                assert_eq!(device.info().device_size(), BLOCK_SIZE as u64 * 1024);
            }

            std::fs::remove_file(&device_name).expect("Failed to remove test file");
        });
    }
}
