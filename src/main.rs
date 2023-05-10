use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};
use dotenv::dotenv;
use ministore::config::EnvironmentVariables;

fn main() -> Result<(), String> {
    // Parse arguments
    let matches = cli();
    let devel = matches.get_flag("devel");
    let test_configfile = matches.get_one::<String>("config");

    // Find run_mode and its config file
    let run_mode = get_run_mode(devel, test_configfile);
    let config_str = run_mode.get_config_str()?;

    // Read envrionment variables
    let environment_variables = get_environment_values();

    // Start ministore
    let start_server = async {
        ministore::telemetry::init_tracing(environment_variables.log_level.as_str())?;
        ministore::start((config_str.as_str(), environment_variables)).await?;
        Ok::<(), String>(())
    };

    // Run server
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to setup tokio runtime");

    runtime.block_on(start_server)?;

    Ok(())
}

fn cli() -> ArgMatches {
    Command::new("MiniStore")
        .version("0.0.1")
        .about("My mini storage service")
        .arg(
            Arg::new("devel")
                .short('d')
                .long("devel")
                .help("Run ministore with development mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .help("Run ministore with test mode with provided config file")
                .long("config"),
        )
        // Only one of these arguments in a group can be used
        .group(ArgGroup::new("run_mode").args(&["devel", "config"]))
        .get_matches()
}

fn get_run_mode(devel: bool, test_configfile: Option<&String>) -> RunMode {
    if test_configfile.is_some() {
        RunMode::Test(test_configfile.unwrap().clone())
    } else if devel == true {
        RunMode::Development
    } else {
        RunMode::Production
    }
}

#[derive(Debug, PartialEq)]
pub enum RunMode {
    Development,
    Production,
    /// In test mode, configuration file for this test should be provided
    Test(String),
}

impl RunMode {
    fn get_config_str(&self) -> Result<String, String> {
        let filename = match self {
            RunMode::Development => "config/development.toml",
            RunMode::Production => "config/production.toml",
            RunMode::Test(file) => file,
        };
        std::fs::read_to_string(filename).map_err(|e| e.to_string())
    }
}

pub fn get_environment_values() -> EnvironmentVariables {
    dotenv().ok();

    let server_addr =
        std::env::var("MINISTORE_SERVER_ADDR").expect("MINISTORE_SERVER_ADDR should be set");
    let server_port =
        std::env::var("MINISTORE_SERVER_PORT").expect("MINISTORE_SERVER_PORT should be set");
    let log_level = std::env::var("RUST_LOG").unwrap_or("INFO".to_string());

    EnvironmentVariables {
        server_addr,
        server_port,
        log_level,
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;
    use tracing_test::traced_test;

    use super::*;

    #[traced_test]
    #[test]
    fn ministore_should_run_with_development_mode_when_devel_set_true() {
        let run_mode = get_run_mode(true, None);

        assert_eq!(run_mode, RunMode::Development);
    }

    #[traced_test]
    #[test]
    fn ministore_should_run_with_test_mode_when_test_name_provided() {
        let test_config = "config/production.toml".to_string(); // temporally use exisiting config file name
        let run_mode = get_run_mode(false, Some(&test_config));

        assert_eq!(run_mode, RunMode::Test(test_config));
    }

    #[traced_test]
    #[test]
    fn ministore_should_run_with_proper_config_file_for_each_run_mode() {
        // Development mode
        let run_mode = RunMode::Development;
        let config_str = run_mode.get_config_str().unwrap();
        let expected_config_str = std::fs::read_to_string("config/development.toml").unwrap();
        assert_eq!(config_str, expected_config_str);

        // Production mode
        let run_mode = RunMode::Production;
        let config_str = run_mode.get_config_str().unwrap();
        let expected_config_str = std::fs::read_to_string("config/production.toml").unwrap();
        assert_eq!(config_str, expected_config_str);

        // Test mode
        let test_config_str = r#"[log]
    level = "debug"

[devices]
    use_fake = true
    fake_device_location = "."
    fake_device_type = "SimpleFake"
        "#;

        let mut test_configfile = std::fs::File::create(
            "ministore_should_run_with_proper_config_file_for_each_run_mode.toml",
        )
        .unwrap();
        test_configfile
            .write_all(test_config_str.as_bytes())
            .unwrap();

        let run_mode = RunMode::Test(
            "ministore_should_run_with_proper_config_file_for_each_run_mode.toml".to_string(),
        );
        let config_str = run_mode.get_config_str().unwrap();
        assert_eq!(config_str, test_config_str);

        std::fs::remove_file("ministore_should_run_with_proper_config_file_for_each_run_mode.toml")
            .unwrap();
    }
}
