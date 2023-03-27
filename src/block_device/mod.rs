use self::device_info::DeviceInfo;

mod device_info;
mod simple_fake_device;

pub const BLOCK_SIZE: usize = 4096;
pub const UNMAP_BLOCK: DataBlock = [0xF; BLOCK_SIZE];
type DataBlock = [u8; BLOCK_SIZE];

pub trait BlockDevice {
    fn info(&self) -> &DeviceInfo;
    fn write(&mut self, lba: u64, num_blocks: u64, buffer: Vec<DataBlock>) -> Result<(), String>;
    fn read(&self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String>;
}
