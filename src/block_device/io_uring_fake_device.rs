use super::{data_type::DataBlock, device_info::DeviceInfo, BlockDevice};
use crate::block_device::data_type::{BLOCK_SIZE, UNMAP_BLOCK};
use std::io::{Seek, Write};
use std::os::fd::AsRawFd;

const URING_SIZE: u32 = 8;

#[cfg(target_os = "linux")]
pub struct IoUringFakeDevice {
    device_info: DeviceInfo,
    ring: io_uring::IoUring,
}

#[cfg(target_os = "linux")]
impl IoUringFakeDevice {
    pub fn new(name: String, size: u64) -> Result<Self, String> {
        let device_info = DeviceInfo::new(name, size)?;
        let ring = io_uring::IoUring::new(URING_SIZE).map_err(|e| e.to_string())?;

        let filename = device_info.name();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(filename)
            .map_err(|e| e.to_string())?;

        for lba in 0..device_info.num_blocks() {
            file.seek(std::io::SeekFrom::Start(lba * BLOCK_SIZE as u64))
                .map_err(|e| e.to_string())?;
            file.write_all(&UNMAP_BLOCK.0).map_err(|e| e.to_string())?;
        }

        Ok(Self { device_info, ring })
    }

    fn is_valid_range(&self, lba: u64, num_blocks: u64) -> bool {
        if num_blocks == 0 || lba + num_blocks > self.device_info.num_blocks() {
            false
        } else {
            true
        }
    }
}

#[cfg(target_os = "linux")]
impl BlockDevice for IoUringFakeDevice {
    fn info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn write(&mut self, lba: u64, num_blocks: u64, buffer: Vec<DataBlock>) -> Result<(), String> {
        if self.is_valid_range(lba, num_blocks) == false {
            return Err("Invalid lba ranges".to_string());
        }

        if (buffer.len() as u64) < num_blocks {
            return Err("Not enough buffers provided".to_string());
        }

        let filename = self.device_info.name();
        let fd = std::fs::OpenOptions::new()
            .write(true)
            .open(filename)
            .map_err(|e| e.to_string())?;

        let mut flatten: Vec<u8> = buffer
            .iter()
            .map(|d| d.0.to_vec())
            .into_iter()
            .flatten()
            .collect();

        let write_e = io_uring::opcode::Write::new(
            io_uring::types::Fd(fd.as_raw_fd()),
            flatten.as_mut_ptr(),
            flatten.len() as _,
        )
        .offset(
            (lba * BLOCK_SIZE as u64)
                .try_into()
                .map_err(|e| format!("{:?}", e))?,
        )
        .build();

        unsafe {
            self.ring
                .submission()
                .push(&write_e)
                .map_err(|e| e.to_string())?;
        }

        self.ring.submit_and_wait(1).map_err(|e| e.to_string())?;

        if let Some(cqe) = self.ring.completion().next() {
            let result = cqe.result();
            if result == 0 {
                Ok(())
            } else {
                Err(format!("Write failed: err:{}", cqe.result()))
            }
        } else {
            Err(format!("Cannot get completion"))
        }
    }

    fn read(&mut self, lba: u64, num_blocks: u64) -> Result<Vec<DataBlock>, String> {
        if self.is_valid_range(lba, num_blocks) == false {
            return Err("Invalid lba ranges".to_string());
        }

        let filename = self.device_info.name();
        let fd = std::fs::OpenOptions::new()
            .read(true)
            .open(filename)
            .map_err(|e| e.to_string())?;

        let size = BLOCK_SIZE * num_blocks as usize;
        let mut flatten_buf = vec![0u8; size];
        let read_e = io_uring::opcode::Read::new(
            io_uring::types::Fd(fd.as_raw_fd()),
            flatten_buf.as_mut_ptr(),
            flatten_buf.len() as _,
        )
        .offset(
            (lba * BLOCK_SIZE as u64)
                .try_into()
                .map_err(|e| format!("{:?}", e))?,
        )
        .build();

        unsafe {
            self.ring
                .submission()
                .push(&read_e)
                .map_err(|e| e.to_string())?;
        }

        self.ring.submit_and_wait(1).map_err(|e| e.to_string())?;

        if let Some(cqe) = self.ring.completion().next() {
            let result = cqe.result();
            if result == 0 {
                let mut buffer: Vec<DataBlock> = Vec::new();
                for chunk in flatten_buf.chunks_exact(BLOCK_SIZE) {
                    let mut block = [0u8; BLOCK_SIZE];
                    block.copy_from_slice(chunk);
                    buffer.push(DataBlock(block));
                }

                Ok(buffer)
            } else {
                Err(format!("Write failed: err:{}", cqe.result()))
            }
        } else {
            Err(format!("Cannot get completion"))
        }
    }
    fn load(&mut self) -> Result<(), String> {
        // Do nothing as data will be read from the file
        Ok(())
    }
    fn flush(&mut self) -> Result<(), String> {
        // Do nothing as data is saved in file
        Ok(())
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
