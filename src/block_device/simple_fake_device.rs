use super::data_type::{DataBlock, UNMAP_BLOCK};
use super::{device_info::DeviceInfo, BlockDevice};
use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, path::Path};

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

pub struct SimpleFakeDevice {
    device_info: DeviceInfo,
    data: Data,
}
impl std::fmt::Debug for SimpleFakeDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleFakeDevice")
            .field("device_info", &self.device_info)
            .finish()
    }
}

impl SimpleFakeDevice {
    pub fn new(name: String, size: u64) -> Result<Self, String> {
        let device_info = DeviceInfo::new(name, size)?;
        let num_blocks = device_info.num_blocks();
        Ok(SimpleFakeDevice {
            device_info: device_info,
            data: Data::new(num_blocks as usize),
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
            self.data.0[current_lba] = buffer[offset as usize];
        }

        Ok(())
    }

    fn read(&mut self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String> {
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

    fn info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn load(&mut self) -> Result<(), String> {
        let filename = self.device_info.name().clone();
        let path = Path::new(&filename);

        if !path.exists() {
            return Err("No files to load".to_string());
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .map_err(|e| e.to_string())?;

        let loaded_data = bincode::deserialize_from(&mut file).map_err(|e| e.to_string())?;
        self.data = loaded_data;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        let filename = self.device_info.name().clone();
        let path = Path::new(&filename);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
            .map_err(|e| e.to_string())?;

        bincode::serialize_into(&mut file, &self.data).map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::data_type::BLOCK_SIZE;
    use super::*;

    #[test]
    fn data_should_be_loaded_from_the_file() {
        {
            let mut device = SimpleFakeDevice::new(
                "data_should_be_loaded_from_the_file".to_string(),
                BLOCK_SIZE as u64 * 1024,
            )
            .expect("Failed to create fake device");

            let mut test_data = Data::new(1024);
            for lba in 0..1024 {
                test_data.0[lba] = DataBlock([lba as u8; BLOCK_SIZE]);
            }

            device.write(0, 1024, test_data.clone().0).unwrap();
            assert_eq!(device.flush().is_ok(), true);
        }

        {
            let mut device = SimpleFakeDevice::new(
                "data_should_be_loaded_from_the_file".to_string(),
                BLOCK_SIZE as u64 * 1024,
            )
            .expect("Failed to create fake device");

            let read_before_load = device.read(0, 1024).unwrap();
            for lba in 0..1024 {
                assert_eq!(read_before_load[lba], UNMAP_BLOCK);
            }

            device.load().expect("Failed to load data");

            let read_after_load = device.read(0, 1024).unwrap();
            for lba in 0..1024 {
                assert_eq!(read_after_load[lba], DataBlock([lba as u8; BLOCK_SIZE]));
            }
        }

        std::fs::remove_file("data_should_be_loaded_from_the_file")
            .expect("Failed to remove test file");
    }

    #[test]
    fn load_should_fail_when_there_is_no_file() {
        let mut device = SimpleFakeDevice::new(
            "load_should_fail_when_there_is_no_file".to_string(),
            BLOCK_SIZE as u64 * 1024,
        )
        .expect("Failed to create fake device");

        assert_eq!(device.load().is_err(), true);
    }
}
