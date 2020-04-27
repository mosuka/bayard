use clap::{App, AppSettings, Arg, SubCommand};

use crate::cli::start::run_start_cli;

pub fn run_root_cli() -> Result<(), std::io::Error> {
    let app = App::new(crate_name!())
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Manage Bayard REST server")
        .help_message("Prints help information.")
        .version_message("Prints version information.")
        .version_short("v")
        .subcommand(
            SubCommand::with_name("start")
                .name("start")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Start REST server")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("HOST")
                        .help("Node address.")
                        .short("H")
                        .long("host")
                        .value_name("HOST")
                        .default_value("0.0.0.0")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("PORT")
                        .help("HTTP service port number.")
                        .short("p")
                        .long("port")
                        .value_name("PORT")
                        .default_value("8000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("SERVER")
                        .help("Index service address.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("0.0.0.0:5000")
                        .takes_value(true),
                ),
        )
        .get_matches();

    let (subcommand, some_options) = app.subcommand();
    let options = some_options.unwrap();
    let run_cli = match subcommand {
        "start" => run_start_cli,
        _ => panic!("Subcommand {} is unknown", subcommand),
    };

    run_cli(options)
}
