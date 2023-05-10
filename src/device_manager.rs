use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::block_device::{create_block_device, BlockDevice};
use crate::block_device_common::data_type::DataBlock;
use crate::block_device_common::str_to_block_device_type;
use crate::config::DeviceConfig;

pub struct DeviceManager {
    devices: HashMap<String, Box<dyn BlockDevice>>,
    config: DeviceConfig,
    dir: PathBuf, // This is to store PathBuf retrieved from the config.fake_device_location
}

impl DeviceManager {
    pub fn new(config: &DeviceConfig) -> Result<Self, String> {
        let mut device_manager = DeviceManager {
            devices: HashMap::new(),
            config: config.clone(),
            dir: PathBuf::from(config.fake_device_location.as_str()),
        };

        if device_manager.config.use_fake == true {
            device_manager.load_devices()?;
        }

        Ok(device_manager)
    }

    fn load_devices(&mut self) -> Result<(), String> {
        if self.dir.exists() == false || self.dir.is_dir() == false {
            std::fs::create_dir(&self.dir).map_err(|e| e.to_string())?;
        }

        for file in fs::read_dir(&self.dir).unwrap() {
            let file = file.map_err(|e| e.to_string())?;

            let device_type = str_to_block_device_type(self.config.fake_device_type.as_str())?;
            let device_name = file
                .file_name()
                .into_string()
                .map_err(|e| format!("{:?}", e))?;

            let mut fake_device =
                create_block_device(device_type, device_name.clone(), 0, self.dir.clone())?;

            fake_device.load()?;

            self.devices.insert(device_name.clone(), fake_device);
        }

        Ok(())
    }

    pub fn create_fake_device(
        &mut self,
        device_name: &String,
        device_size: u64,
    ) -> Result<(), String> {
        if self.config.use_fake == false {
            return Err(format!("use_fake is false in configuration"));
        }

        if self.devices.contains_key(device_name) {
            return Err(format!("Device already exists, name:{}", device_name));
        }

        let device_type = str_to_block_device_type(self.config.fake_device_type.as_str())?;
        let device = create_block_device(
            device_type,
            device_name.clone(),
            device_size,
            self.dir.clone(),
        )?;

        self.devices.insert(device_name.clone(), device);

        Ok(())
    }

    pub fn delete_fake_device(&mut self, device_name: &String) -> Result<(), String> {
        if self.config.use_fake == false {
            return Err(format!("use_fake is false in configuration"));
        }

        match self.devices.remove(device_name) {
            Some(device) => {
                let device_name = device.info().name();
                let mut device_filepath = self.dir.clone();
                device_filepath.push(&device_name);
                std::fs::remove_file(device_filepath)
                    .map_err(|e| e.to_string())
                    .unwrap_or(());
            }
            None => return Err(format!("No such device, name:{}", device_name)),
        };

        Ok(())
    }

    pub fn list_fake_devices(&self) -> Result<Vec<(String, u64)>, String> {
        if self.config.use_fake == false {
            Err(format!("use_fake is false in configuration"))
        } else {
            Ok(self
                .devices
                .iter()
                .map(|dev| (dev.0.clone(), dev.1.info().device_size()))
                .collect())
        }
    }

    pub fn write(
        &mut self,
        device_name: &String,
        lba: u64,
        num_blocks: u64,
        blocks: Vec<DataBlock>,
    ) -> Result<(), String> {
        if self.devices.contains_key(device_name) == false {
            return Err("No such device".to_string());
        }

        let device = self.devices.get_mut(device_name).unwrap();
        device.write(lba, num_blocks, blocks)
    }

    pub fn read(
        &mut self,
        device_name: &String,
        lba: u64,
        num_blocks: u64,
    ) -> Result<Vec<DataBlock>, String> {
        if self.devices.contains_key(device_name) == false {
            return Err("No such device".to_string());
        }

        let device = self.devices.get_mut(device_name).unwrap();
        device.read(lba, num_blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::humansize_to_integer;
    use tracing_test::traced_test;

    fn test_device_config(dirname: &str) -> DeviceConfig {
        DeviceConfig {
            use_fake: true,
            fake_device_location: dirname.to_string(),
            fake_device_type: "SimpleFake".to_string(),
        }
    }

    #[traced_test]
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

    #[traced_test]
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
