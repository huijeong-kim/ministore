use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};

fn main() -> Result<(), String> {
    let matches = cli();

    let devel = matches.get_flag("devel");
    let test_configfile = matches.get_one::<String>("config");
    ministore::start(devel, test_configfile)?;

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
