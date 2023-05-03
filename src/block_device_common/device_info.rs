use super::data_type::BLOCK_SIZE;
use super::BlockDeviceType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    device_type: BlockDeviceType,
    device_name: String,
    device_size: u64,
    num_blocks: u64,
}

impl DeviceInfo {
    pub fn new(
        device_type: BlockDeviceType,
        device_name: String,
        device_size: u64,
    ) -> Result<Self, String> {
        match device_size % BLOCK_SIZE as u64 != 0 {
            true => return Err("Unaligned device size".to_string()),
            false => (),
        }

        Ok(Self {
            device_type,
            device_name,
            device_size,
            num_blocks: device_size / BLOCK_SIZE as u64,
        })
    }

    pub fn name(&self) -> &String {
        &self.device_name
    }

    pub fn device_size(&self) -> u64 {
        self.device_size
    }

    pub fn num_blocks(&self) -> u64 {
        self.num_blocks
    }

    pub fn device_type(&self) -> BlockDeviceType {
        self.device_type.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_device_info_should_success() {
        let device_name = "create_device_info";
        let block_size = 4096;
        let num_blocks = 100;
        let device_size = block_size * num_blocks;

        let device_info = DeviceInfo::new(
            BlockDeviceType::SimpleFakeDevice,
            device_name.to_string(),
            device_size,
        );
        assert_eq!(device_info.is_ok(), true);

        assert_eq!(
            device_info.as_ref().unwrap().name(),
            &device_name.to_string()
        );
        assert_eq!(device_info.as_ref().unwrap().device_size(), device_size);
        assert_eq!(
            device_info.as_ref().unwrap().num_blocks(),
            device_size / block_size
        );
        assert_eq!(
            device_info.as_ref().unwrap().device_type(),
            BlockDeviceType::SimpleFakeDevice
        );
    }

    #[test]
    fn create_device_info_with_unaligned_size_should_fail() {
        let device_name = "create_device_info";
        let device_size = 500000;

        let device_info = DeviceInfo::new(
            BlockDeviceType::SimpleFakeDevice,
            device_name.to_string(),
            device_size,
        );

        assert_eq!(device_info.is_err(), true);
    }
}
