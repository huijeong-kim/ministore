use super::{data_type::DataBlock, device_info::DeviceInfo, BlockDevice};
use crate::block_device::data_type::{BLOCK_SIZE, UNMAP_BLOCK};
use std::io::{Seek, Write};
use std::os::fd::AsRawFd;

const URING_SIZE: u32 = 8;

#[cfg(target_os = "linux")]
pub struct IoUringFakeDevice {
}

#[cfg(target_os = "linux")]
impl IoUringFakeDevice {
    pub fn new(name: String, size: u64) -> Result<Self, String> {
        todo!()
    }
}

#[cfg(target_os = "linux")]
impl BlockDevice for IoUringFakeDevice {
    fn info(&self) -> &DeviceInfo {
        todo!()
    }

    fn write(&mut self, lba: u64, num_blocks: u64, buffer: Vec<DataBlock>) -> Result<(), String> {
        todo!()
    }

    fn read(&mut self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String> {
        todo!()
    }

    fn load(&mut self) -> Result<(), String> {
        todo!()
    }
    fn flush(&mut self) -> Result<(), String> {
        todo!()
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use io_uring::{opcode, types, IoUring};
    use std::fs;
    use std::os::unix::io::AsRawFd;
    use std::panic;
    use std::path::Path;

    fn panic_hook(info: &panic::PanicInfo<'_>) {
        println!("Panic occurred: {:?}", info);
        let path = Path::new("text.txt");
        if path.try_exists().unwrap() {
            fs::remove_file(path).unwrap();
        }
    }

    /// This is a simple example of using uring on linux machine
    #[test]
    pub fn simple_uring_test_on_linux() {
        panic::set_hook(Box::new(panic_hook));
        let mut ring = IoUring::new(8).expect("Failed to create IoUring");

        let file_name = "text.txt";
        let fd = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_name.clone())
            .expect("Failed to open file");
        // Write data to the file
        {
            let mut buf: [u8; 1024] = [0xA; 1024];
            let write_e =
                opcode::Write::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
                    .build();

            unsafe {
                ring.submission()
                    .push(&write_e)
                    .expect("submission queue is full");
            }

            ring.submit_and_wait(1)
                .expect("Failed to submit write request to ring");
            let cqe = ring.completion().next().expect("completion queue is empty");
            assert!(cqe.result() >= 0, "write error: {}", cqe.result());
        }

        // Read data from the file
        {
            let mut buf = [0u8; 1024];
            let read_e =
                opcode::Read::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
                    .build();

            unsafe {
                ring.submission()
                    .push(&read_e)
                    .expect("submission queue is full");
            }

            ring.submit_and_wait(1)
                .expect("Failed to submit read request to ring");
            let cqe = ring.completion().next().expect("completion queue is empty");
            assert!(cqe.result() >= 0, "read error: {}", cqe.result());

            assert_eq!(buf, [0xA; 1024]);
            fs::remove_file(file_name).unwrap();
        }
    }
}
