use super::{BlockDevice, BlockDeviceType};
use crate::block_device_common::data_type::{DataBlock, UNMAP_BLOCK};
use crate::block_device_common::device_info::DeviceInfo;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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

#[derive(Serialize, Deserialize)]
pub struct SimpleFakeDevice {
    device_info: DeviceInfo,
    data: Data,
    filepath: PathBuf,
}
impl std::fmt::Debug for SimpleFakeDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleFakeDevice")
            .field("device_info", &self.device_info)
            .finish()
    }
}

impl SimpleFakeDevice {
    pub fn new(name: String, size: u64, filepath: PathBuf) -> Result<Self, String> {
        let device_info = DeviceInfo::new(BlockDeviceType::SimpleFakeDevice, name, size)?;
        let num_blocks = device_info.num_blocks();

        let mut device = SimpleFakeDevice {
            device_info,
            data: Data::new(num_blocks as usize),
            filepath,
        };

        if Path::new(device.device_info.name()).exists() == false {
            device.flush()?;
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

        let mut path = self.filepath.clone();
        path.push(&filename);

        if !path.exists() {
            return Err("No files to load".to_string());
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| e.to_string())?;

        let loaded_data: SimpleFakeDevice =
            bincode::deserialize_from(&mut file).map_err(|e| e.to_string())?;

        if loaded_data.device_info.device_type() != BlockDeviceType::SimpleFakeDevice {
            return Err("Loaded device file type is invalid".to_string());
        }

        self.device_info = loaded_data.device_info;
        self.data = loaded_data.data;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        let filename = self.device_info.name().clone();

        let mut path = self.filepath.clone();
        path.push(&filename);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
            .map_err(|e| e.to_string())?;

        bincode::serialize_into(&mut file, &self).map_err(|e| e.to_string())?;

        Ok(())
    }
}
