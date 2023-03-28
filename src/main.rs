use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};
use ministore::config::RunMode;

fn main() -> Result<(), String> {
    let matches = cli();

    let devel = matches.get_flag("devel");
    let test_name = matches.get_one::<String>("test");
    let run_mode = get_run_mode(devel, test_name);

    let config = ministore::config::get_config(run_mode)?;

    // This is simple, temporal test
    test_create_fake_devices(config.devices);

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
            Arg::new("test")
                .short('t')
                .help("Run ministore with test mode with test name")
                .long("test"),
        )
        // Only one of these arguments in a group can be used
        .group(ArgGroup::new("run_mode").args(&["devel", "test"]))
        .get_matches()
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

pub fn test_create_fake_devices(config: ministore::config::DeviceConfig) {
    println!("Device configuration:");
    println!("{:#?}", config);

    if config.use_fake == true {
        let devices = ministore::block_device::create_fake_devices_from_list(&config);
        println!("Fake devices crated:");
        println!("{:#?}", devices);
    }
}
