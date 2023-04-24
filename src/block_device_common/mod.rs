pub mod data_type;
pub mod device_info;

use strum_macros::{Display, EnumIter};

#[derive(Debug, EnumIter, Clone, Display, PartialEq)]
pub enum BlockDeviceType {
    SimpleFakeDevice,
    IoUringFakeDevice,
}

pub fn i32_to_block_device_type(value: i32) -> Result<BlockDeviceType, String> {
    match value {
        0 => Ok(BlockDeviceType::SimpleFakeDevice),
        1 => Ok(BlockDeviceType::IoUringFakeDevice),
        _ => Err(format!("Wrong device type, type={}", value)),
    }
}

impl From<BlockDeviceType> for i32 {
    fn from(value: BlockDeviceType) -> Self {
        match value {
            BlockDeviceType::SimpleFakeDevice => 0,
            BlockDeviceType::IoUringFakeDevice => 1,
        }
    }
}
