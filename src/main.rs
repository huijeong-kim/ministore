use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};

fn main() -> Result<(), String> {
    let matches = cli();

    let devel = matches.get_flag("devel");
    let test_name = matches.get_one::<String>("test");
    ministore::start(devel, test_name)?;   

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
