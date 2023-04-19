pub mod block_device;
pub mod config;
pub mod grpc_server;
pub mod utils;

use self::config::RunMode;

pub fn start(devel: bool, test_name: Option<&String>) -> Result<RunMode, String> {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ministore_should_run_with_development_mode_when_devel_set_true() {
        let result = start(true, None);

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), RunMode::Development);
    }

    #[test]
    fn ministore_should_run_with_test_mode_when_test_name_provided() {
        let test_name = "production".to_string(); // temporally use exisiting config file name
        let result = start(false, Some(&test_name));

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), RunMode::Custom(test_name));
    }
}
