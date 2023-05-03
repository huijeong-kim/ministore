pub mod data_type;
pub mod device_info;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[derive(Debug, EnumIter, Clone, Display, PartialEq, Serialize, Deserialize)]
pub enum BlockDeviceType {
    SimpleFakeDevice,
    IoUringFakeDevice,
    AsyncSimpleFakeDevice,
}

impl BlockDeviceType {
    pub fn is_async(&self) -> bool {
        match &self {
            BlockDeviceType::SimpleFakeDevice => false,
            BlockDeviceType::IoUringFakeDevice => false,
            BlockDeviceType::AsyncSimpleFakeDevice => true,
        }
    }

    pub fn is_sync(&self) -> bool {
        !self.is_async()
    }
}

pub fn i32_to_block_device_type(value: i32) -> Result<BlockDeviceType, String> {
    match value {
        0 => Ok(BlockDeviceType::SimpleFakeDevice),
        1 => Ok(BlockDeviceType::IoUringFakeDevice),
        2 => Ok(BlockDeviceType::AsyncSimpleFakeDevice),
        _ => Err(format!("Wrong device type, type={}", value)),
    }
}

impl From<BlockDeviceType> for i32 {
    fn from(value: BlockDeviceType) -> Self {
        match value {
            BlockDeviceType::SimpleFakeDevice => 0,
            BlockDeviceType::IoUringFakeDevice => 1,
            BlockDeviceType::AsyncSimpleFakeDevice => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn all_block_device_type_should_be_converted_from_or_into_i32() {
        let mut type_num: i32 = 0;
        for device_type in BlockDeviceType::iter() {
            assert_eq!(device_type, i32_to_block_device_type(type_num).unwrap());
            assert_eq!(i32::from(device_type), type_num);

            type_num += 1;
        }
    }

    #[test]
    fn all_block_device_type_should_tell_if_it_is_sync_or_async() {
        assert_eq!(BlockDeviceType::SimpleFakeDevice.is_sync(), true);
        assert_eq!(BlockDeviceType::SimpleFakeDevice.is_async(), false);

        assert_eq!(BlockDeviceType::AsyncSimpleFakeDevice.is_sync(), false);
        assert_eq!(BlockDeviceType::AsyncSimpleFakeDevice.is_async(), true);
    }
}
