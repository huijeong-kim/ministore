pub mod async_block_device;
pub mod block_device;
pub mod block_device_common;
pub mod config;
pub mod device_manager;
pub mod grpc_server;
pub mod utils;

#[derive(Debug, PartialEq)]
pub enum RunMode {
    Development,
    Production,
    /// In test mode, configuration file for this test should be provided
    Test(String),
}

pub fn start(run_mode: RunMode) -> Result<(), String> {
    let config = config::get_config(&run_mode)?;
    let environment_variables = config::get_environment_values();

    println!("run_mode: {:?}", run_mode);
    println!("config: {:#?}", config);
    println!("environment_variables: {:?}", environment_variables);

    // Do something here..

    Ok(())
}

pub fn get_run_mode(devel: bool, test_name: Option<&String>) -> RunMode {
    if test_name.is_some() {
        RunMode::Test(test_name.unwrap().clone())
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
        let run_mode = get_run_mode(true, None);

        assert_eq!(run_mode, RunMode::Development);
    }

    #[test]
    fn ministore_should_run_with_test_mode_when_test_name_provided() {
        let test_config = "config/production.toml".to_string(); // temporally use exisiting config file name
        let run_mode = get_run_mode(false, Some(&test_config));

        assert_eq!(run_mode, RunMode::Test(test_config));
    }
}
