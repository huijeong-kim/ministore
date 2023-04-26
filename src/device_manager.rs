use std::collections::HashMap;

use crate::block_device::{create_block_device, BlockDevice};
use crate::block_device_common::i32_to_block_device_type;

#[derive(Default)]
pub struct DeviceManager {}

impl DeviceManager {
    pub fn create_fake_device(
        &mut self,
        device_type: i32,
        device_name: &String,
        device_size: u64,
    ) -> Result<(), String> {
        todo!()
    }

    pub fn delete_fake_device(&mut self, device_name: &String) -> Result<(), String> {
        todo!()
    }

    pub fn list_fake_devices(&self) -> Result<Vec<(String, u64, i32)>, String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::humansize_to_integer;

    use super::*;

    #[test]
    fn device_manager_can_create_and_delete_device() {
        let mut device_manager = DeviceManager::default();

        // type = SimpleFakeDevice
        // name = "device_manager_can_create_and_delete_device"
        // size = 1MB
        let device_name = "device_manager_can_create_and_delete_device".to_string();

        device_manager
            .create_fake_device(0, &device_name, humansize_to_integer("1M").unwrap())
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
        assert_eq!(devices.get(0).unwrap().2, 0);

        device_manager
            .delete_fake_device(&device_name)
            .expect("Failed to remove fake device");
    }

    #[test]
    fn device_manager_cannot_create_device_with_same_name_twice() {
        let mut device_manager = DeviceManager::default();

        // type = SimpleFakeDevice
        // name = "device_manager_cannot_create_device_with_same_name_twice"
        // size = 1MB
        let device_name = "device_manager_cannot_create_device_with_same_name_twice".to_string();
        device_manager
            .create_fake_device(0, &device_name, humansize_to_integer("1M").unwrap())
            .expect("Failed to create fake device");

        assert_eq!(
            device_manager
                .create_fake_device(0, &device_name, humansize_to_integer("1M").unwrap())
                .is_ok(),
            false
        );

        std::fs::remove_file(device_name.clone()).expect("Failed to remove file");
    }
}
