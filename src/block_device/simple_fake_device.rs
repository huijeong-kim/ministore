use super::{BlockDevice, BlockDeviceType};
use crate::block_device_common::data_type::{DataBlock, UNMAP_BLOCK};
use crate::block_device_common::device_info::DeviceInfo;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fs::OpenOptions, path::Path};

pub struct SimpleFakeDevice {}

impl SimpleFakeDevice {
    pub fn new(name: String, size: u64, filepath: PathBuf) -> Result<Self, String> {
        todo!()
    }
}

impl BlockDevice for SimpleFakeDevice {
    fn write(&mut self, lba: u64, num_blocks: u64, buffer: Vec<DataBlock>) -> Result<(), String> {
        todo!()
    }

    fn read(&mut self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String> {
        todo!()
    }

    fn info(&self) -> &DeviceInfo {
        todo!()
    }

    fn load(&mut self) -> Result<(), String> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), String> {
        todo!()
    }
}
