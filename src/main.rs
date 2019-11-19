extern crate bayard;
#[macro_use]
extern crate clap;

use std::io::Write;

use clap::{App, AppSettings, Arg, SubCommand};

use bayard::cmd::delete::run_delete_cli;
use bayard::cmd::get::run_get_cli;
use bayard::cmd::leave::run_leave_cli;
use bayard::cmd::peers::run_peers_cli;
use bayard::cmd::search::run_search_cli;
use bayard::cmd::serve::run_serve_cli;
use bayard::cmd::set::run_set_cli;

fn main() {
    let app = App::new(crate_name!())
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("serve")
                .name("serve")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Start server")
                .arg(
                    Arg::with_name("ID")
                        .help("The node ID")
                        .short("i")
                        .long("id")
                        .value_name("ID")
                        .default_value("1")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("HOST")
                        .help("The node address")
                        .short("H")
                        .long("host")
                        .value_name("HOST")
                        .default_value("0.0.0.0")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("PORT")
                        .help("The gRPC listen port for client connection")
                        .short("P")
                        .long("port")
                        .value_name("PORT")
                        .default_value("5000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("PEERS")
                        .help("Set peers address separated by `,`, if join to a cluster")
                        .short("p")
                        .long("peers")
                        .value_name("ID=IP:PORT")
                        .multiple(true)
                        .takes_value(true)
                        .use_delimiter(true)
                        .require_delimiter(true)
                        .value_delimiter(","),
                )
                .arg(
                    Arg::with_name("DATA_DIRECTORY")
                        .help("The data directory")
                        .short("d")
                        .long("data-directory")
                        .value_name("DATA_DIRECTORY")
                        .default_value("./data")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("SCHEMA_FILE")
                        .help("The schema file")
                        .short("s")
                        .long("schema-file")
                        .value_name("SCHEMA_FILE")
                        .default_value("./etc/schema.json")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("UNIQUE_KEY_FIELD_NAME")
                        .help("The unique key field name")
                        .short("u")
                        .long("unique-key-field-name")
                        .value_name("UNIQUE_KEY_FIELD_NAME")
                        .default_value("id")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("peers")
                .name("peers")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Get cluster peers")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("The server addresses. Use `,` to separate address. Example: `127.0.0.1:5000,127.0.0.1:5001`")
                        .short("s")
                        .long("servers")
                        .value_name("IP:PORT")
                        .default_value("127.0.0.1:5000")
                        .multiple(true)
                        .use_delimiter(true)
                        .require_delimiter(true)
                        .value_delimiter(",")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("leave")
                .name("leave")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Remove a node from a cluster")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("The server addresses. Use `,` to separate address. Example: `127.0.0.1:5000,127.0.0.1:5001`")
                        .short("s")
                        .long("servers")
                        .value_name("IP:PORT")
                        .default_value("127.0.0.1:5000")
                        .multiple(true)
                        .use_delimiter(true)
                        .require_delimiter(true)
                        .value_delimiter(",")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ID")
                        .help("Node ID to be removed from the cluster")
                        .short("i")
                        .long("id")
                        .value_name("ID")
                        .default_value("1")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("set")
                .name("set")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Index document")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("The server addresses. Use `,` to separate address. Example: `127.0.0.1:5000,127.0.0.1:5001`")
                        .short("s")
                        .long("servers")
                        .value_name("IP:PORT")
                        .default_value("127.0.0.1:5000")
                        .multiple(true)
                        .use_delimiter(true)
                        .require_delimiter(true)
                        .value_delimiter(",")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("KEY")
                        .help("The key")
                        .value_name("KEY")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("VALUE")
                        .help("The value")
                        .value_name("VALUE")
                        .required(true)
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("get")
                .name("get")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Get document")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("The server addresses. Use `,` to separate address. Example: `127.0.0.1:5000,127.0.0.1:5001`")
                        .short("s")
                        .long("servers")
                        .value_name("IP:PORT")
                        .default_value("127.0.0.1:5000")
                        .multiple(true)
                        .use_delimiter(true)
                        .require_delimiter(true)
                        .value_delimiter(",")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("KEY")
                        .help("The key")
                        .value_name("KEY")
                        .required(true)
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("delete")
                .name("delete")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Delete document")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("The server addresses. Use `,` to separate address. Example: `127.0.0.1:5000,127.0.0.1:5001`")
                        .short("s")
                        .long("servers")
                        .value_name("IP:PORT")
                        .default_value("127.0.0.1:5000")
                        .multiple(true)
                        .use_delimiter(true)
                        .require_delimiter(true)
                        .value_delimiter(",")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("KEY")
                        .help("The key")
                        .value_name("KEY")
                        .required(true)
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("search")
                .name("search")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Search documents")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("The server addresses. Use `,` to separate address. Example: `127.0.0.1:5000,127.0.0.1:5001`")
                        .short("s")
                        .long("servers")
                        .value_name("IP:PORT")
                        .default_value("127.0.0.1:5000")
                        .multiple(true)
                        .use_delimiter(true)
                        .require_delimiter(true)
                        .value_delimiter(",")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("QUERY")
                        .help("The query")
                        .value_name("QUERY")
                        .required(true)
                        .takes_value(true),
                )
        )
//        .subcommand(
//            SubCommand::with_name("version")
//                .name("version")
//                .setting(AppSettings::DeriveDisplayOrder)
//                .version(crate_version!())
//                .author(crate_authors!())
//                .about("Show remote server version")
//                .arg(
//                    Arg::with_name("HOST")
//                        .help("The node address")
//                        .short("H")
//                        .long("host")
//                        .default_value("0.0.0.0")
//                        .takes_value(true),
//                )
//                .arg(
//                    Arg::with_name("CLIENT_PORT")
//                        .help("The gRPC listen port for client connection")
//                        .short("c")
//                        .long("client-port")
//                        .default_value("5000")
//                        .takes_value(true),
//                )
//        )
        .get_matches();

    let (subcommand, some_options) = app.subcommand();
    let options = some_options.unwrap();
    let run_cli = match subcommand {
        "serve" => run_serve_cli,
        "peers" => run_peers_cli,
        "leave" => run_leave_cli,
        "set" => run_set_cli,
        "get" => run_get_cli,
        "delete" => run_delete_cli,
        "search" => run_search_cli,
        //        "version" => run_version_cli,
        _ => panic!("Subcommand {} is unknown", subcommand),
    };

    if let Err(ref e) = run_cli(options) {
        let stderr = &mut std::io::stderr();
        let errmsg = "Error writing to stderr";
        writeln!(stderr, "{}", e).expect(errmsg);
        std::process::exit(1);
    }
}
