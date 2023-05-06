pub mod data_type;
pub mod device_info;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[derive(Debug, EnumIter, Clone, Display, PartialEq, Serialize, Deserialize)]
pub enum BlockDeviceType {
    SimpleFakeDevice,
    AsyncSimpleFakeDevice,
    // IoUringFakeDevice,
}

impl BlockDeviceType {
    pub fn is_async(&self) -> bool {
        match &self {
            BlockDeviceType::SimpleFakeDevice => false,
            BlockDeviceType::AsyncSimpleFakeDevice => true,
        }
    }

    pub fn is_sync(&self) -> bool {
        !self.is_async()
    }
}

pub fn str_to_block_device_type(value: &str) -> Result<BlockDeviceType, String> {
    match value {
        "SimpleFake" => Ok(BlockDeviceType::SimpleFakeDevice),
        "AsyncSimpleFake" => Ok(BlockDeviceType::AsyncSimpleFakeDevice),
        _ => Err(format!("Invalid block device type, type={}", value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test should be updated whenever you add new type
    #[test]
    fn all_block_device_type_should_be_converted_from_str() {
        assert_eq!(
            str_to_block_device_type("SimpleFake"),
            Ok(BlockDeviceType::SimpleFakeDevice)
        );
        assert_eq!(
            str_to_block_device_type("AsyncSimpleFake"),
            Ok(BlockDeviceType::AsyncSimpleFakeDevice)
        );
    }

    #[test]
    fn all_block_device_type_should_tell_if_it_is_sync_or_async() {
        assert_eq!(BlockDeviceType::SimpleFakeDevice.is_sync(), true);
        assert_eq!(BlockDeviceType::SimpleFakeDevice.is_async(), false);

        // Add test here when you add new type
    }
}
