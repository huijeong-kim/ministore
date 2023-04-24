pub mod async_block_device;
pub mod block_device;
pub mod block_device_common;
pub mod config;
pub mod device_manager;
pub mod grpc_server;
pub mod utils;

use self::config::RunMode;

pub fn start(devel: bool, test_name: Option<&String>) -> Result<RunMode, String> {
    let run_mode = get_run_mode(devel, test_name);
    let config = config::get_config(&run_mode)?;
    println!("config: {:#?}", config);

    // Do something here..

    Ok(run_mode)
}

fn get_run_mode(devel: bool, test_name: Option<&String>) -> RunMode {
    if test_name.is_some() {
        RunMode::Custom(test_name.unwrap().clone())
    } else if devel == true {
        RunMode::Development
    } else {
        RunMode::Production
    }
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
