use serde::{Deserialize, Serialize};

use self::device_info::DeviceInfo;

mod device_info;
mod simple_fake_device;
mod simple_async_fake_device;

pub const BLOCK_SIZE: usize = 4096;
pub const UNMAP_BLOCK: DataBlock = DataBlock([0xF; BLOCK_SIZE]);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataBlock([u8; BLOCK_SIZE]);

impl Serialize for DataBlock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for DataBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DataBlockVisitor;

        impl<'de> serde::de::Visitor<'de> for DataBlockVisitor {
            type Value = DataBlock;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("byte array of length 4096")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.len() == BLOCK_SIZE {
                    let mut array = [0; BLOCK_SIZE];
                    array.copy_from_slice(v);
                    Ok(DataBlock(array))
                } else {
                    Err(E::invalid_length(v.len(), &self))
                }
            }
        }

        deserializer.deserialize_bytes(DataBlockVisitor)
    }
}

pub trait BlockDevice {
    fn info(&self) -> &DeviceInfo;
    fn write(&mut self, lba: u64, num_blocks: u64, buffer: Vec<DataBlock>) -> Result<(), String>;
    fn read(&self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String>;
}
