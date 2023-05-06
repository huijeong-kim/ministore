use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::block_device::{create_block_device, BlockDevice};
use crate::block_device_common::data_type::DataBlock;
use crate::block_device_common::str_to_block_device_type;
use crate::config::DeviceConfig;

pub struct DeviceManager {}

impl DeviceManager {
    pub fn new(config: &DeviceConfig) -> Result<Self, String> {
        todo!()
    }

    pub fn create_fake_device(
        &mut self,
        device_name: &String,
        device_size: u64,
    ) -> Result<(), String> {
        todo!()
    }

    pub fn delete_fake_device(&mut self, device_name: &String) -> Result<(), String> {
        todo!()
    }

    pub fn list_fake_devices(&self) -> Result<Vec<(String, u64)>, String> {
        todo!()
    }

    pub fn write(
        &mut self,
        device_name: &String,
        lba: u64,
        num_blocks: u64,
        blocks: Vec<DataBlock>,
    ) -> Result<(), String> {
        todo!()
    }

    pub fn read(
        &mut self,
        device_name: &String,
        lba: u64,
        num_blocks: u64,
    ) -> Result<Vec<DataBlock>, String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::humansize_to_integer;

    use super::*;

    fn test_device_config(dirname: &str) -> DeviceConfig {
        DeviceConfig {
            use_fake: true,
            fake_device_location: dirname.to_string(),
            fake_device_type: "SimpleFake".to_string(),
        }
    }
    #[test]
    fn device_manager_can_create_and_delete_device() {
        let testname = "device_manager_can_create_and_delete_device";
        let config = test_device_config(&testname);
        let mut device_manager = DeviceManager::new(&config).unwrap();

        // type = SimpleFakeDevice
        // name = "device_manager_can_create_and_delete_device"
        // size = 1MB
        let device_name = testname.to_string();

        device_manager
            .create_fake_device(&device_name, humansize_to_integer("1M").unwrap())
            .expect("Failed to create fake device");

        let devices = device_manager
            .list_fake_devices()
            .expect("Failed to get device list");
        assert_eq!(devices.len(), 1);
        assert_eq!(devices.get(0).unwrap().0, device_name);
        assert_eq!(
            devices.get(0).unwrap().1,
            humansize_to_integer("1M").unwrap()
        );

        device_manager
            .delete_fake_device(&device_name)
            .expect("Failed to remove fake device");

        std::fs::remove_dir(&testname).expect("Failed to remove directory");
    }

    #[test]
    fn device_manager_cannot_create_device_with_same_name_twice() {
        let testname = "device_manager_cannot_create_device_with_same_name_twice";
        let config = test_device_config(&testname);
        let mut device_manager =
            DeviceManager::new(&config).expect("Failed to create device manager");

        // type = SimpleFakeDevice
        // name = "device_manager_cannot_create_device_with_same_name_twice"
        // size = 1MB
        let device_name = testname.to_string();
        device_manager
            .create_fake_device(&device_name, humansize_to_integer("1M").unwrap())
            .expect("Failed to create fake device");

        assert_eq!(
            device_manager
                .create_fake_device(&device_name, humansize_to_integer("1M").unwrap())
                .is_ok(),
            false
        );

        std::fs::remove_dir_all(&testname).expect("Failed to remove directory");
    }
}
