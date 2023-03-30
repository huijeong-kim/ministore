pub struct IoUringFakeDevice {

}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use io_uring::{opcode, types, IoUring};
    use std::os::unix::io::AsRawFd;
    use std::fs;
    
    #[test]
    pub fn temp_uring_test_on_linux() {
        // FIXME: it's failing now...
        let mut ring = IoUring::new(8).expect("Failed to create IoUring");

        let fd = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("test.txt")
            .expect("Failed to open file");

        // Write data to the file
        {
            let mut buf = vec![0x0; 1024];
            let write_e = opcode::Write::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
                .offset64(0)
                .build()
                .user_data(0x42);

            unsafe {
                ring.submission()
                    .push(&write_e)
                    .expect("submission queue is full");
            }

            ring.submit_and_wait(1).expect("Failed to submit write request to ring");
            let cqe = ring.completion().next().expect("completion queue is empty");
            assert!(cqe.result() >= 0, "write error: {}", cqe.result());
        }

        // Read data from the file
        {
            let mut buf = vec![0x0; 1024];
            let read_e = opcode::Read::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
                .offset64(0)
                .build();

            unsafe {
                ring.submission()
                    .push(&read_e)
                    .expect("submission queue is full");
            }

            ring.submit_and_wait(1).expect("Failed to submit read request to ring");
            let cqe = ring.completion().next().expect("completion queue is empty");
            assert!(cqe.result() >= 0, "read error: {}", cqe.result());
            assert_eq!(cqe.user_data(), 0x42);
        }
    }
}