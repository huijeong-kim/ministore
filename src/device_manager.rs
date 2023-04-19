use std::collections::HashMap;

use crate::block_device::{create_block_device, i32_to_block_device_type, BlockDevice};

#[derive(Default)]
pub struct DeviceManager {
    devices: HashMap<String, Box<dyn BlockDevice>>,
}

impl DeviceManager {
    pub fn create_fake_device(
        &mut self,
        device_type: i32,
        device_name: &String,
        device_size: u64,
    ) -> Result<(), String> {
        if self.devices.contains_key(device_name) {
            return Err(format!("Device already exists, name:{}", device_name));
        }

        let device_type = i32_to_block_device_type(device_type)?;
        let device = create_block_device(device_type, device_name.clone(), device_size)?;

        self.devices.insert(device_name.clone(), device);

        Ok(())
    }

    pub fn delete_fake_device(&mut self, device_name: &String) -> Result<(), String> {
        match self.devices.remove(device_name) {
            Some(device) => {
                let device_name = device.info().name();
                std::fs::remove_file(&device_name).map_err(|e| e.to_string())?;
            }
            None => return Err(format!("No such device, name:{}", device_name)),
        };

        Ok(())
    }

    pub fn list_fake_devices(&self) -> Result<Vec<(String, u64, i32)>, String> {
        Ok(self
            .devices
            .iter()
            .map(|dev| {
                (
                    dev.0.clone(),
                    dev.1.info().device_size(),
                    dev.1.info().device_type().into(),
                )
            })
            .collect())
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
