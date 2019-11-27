extern crate bayard;
#[macro_use]
extern crate clap;

use std::io::Write;

use clap::{App, AppSettings, Arg, SubCommand};

use bayard::cmd::commit::run_commit_cli;
use bayard::cmd::delete::run_delete_cli;
use bayard::cmd::gateway::run_gateway_cli;
use bayard::cmd::get::run_get_cli;
use bayard::cmd::leave::run_leave_cli;
use bayard::cmd::merge::run_merge_cli;
use bayard::cmd::metrics::run_metrics_cli;
use bayard::cmd::peers::run_peers_cli;
use bayard::cmd::probe::run_probe_cli;
use bayard::cmd::rollback::run_rollback_cli;
use bayard::cmd::schedule::run_schedule_cli;
use bayard::cmd::schema::run_schema_cli;
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
            SubCommand::with_name("probe")
                .name("probe")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Probe a server")
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
            SubCommand::with_name("metrics")
                .name("metrics")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Get metrics")
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
            SubCommand::with_name("commit")
                .name("commit")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Commit index")
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
            SubCommand::with_name("rollback")
                .name("rollback")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Rollback index")
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
            SubCommand::with_name("merge")
                .name("merge")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Merge index")
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
        .subcommand(
            SubCommand::with_name("schema")
                .name("schema")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Get schema")
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
            SubCommand::with_name("schedule")
                .name("schedule")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Schedule jobs")
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
                    Arg::with_name("COMMIT_SCHEDULE")
                        .help("The commit schedule")
                        .short("c")
                        .long("commit")
                        .value_name("COMMIT_SCHEDULE")
                        .default_value("0/10 * * * * * *")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("MERGE_SCHEDULE")
                        .help("The merge schedule")
                        .short("m")
                        .long("merge")
                        .value_name("MERGE_SCHEDULE")
                        .default_value("0 0 2 * * * *")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("gateway")
                .name("gateway")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Schedule jobs")
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
                        .help("The HTTP listen port for client connection")
                        .short("P")
                        .long("port")
                        .value_name("PORT")
                        .default_value("8000")
                        .takes_value(true),
                )
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
        .get_matches();

    let (subcommand, some_options) = app.subcommand();
    let options = some_options.unwrap();
    let run_cli = match subcommand {
        "serve" => run_serve_cli,
        "probe" => run_probe_cli,
        "peers" => run_peers_cli,
        "metrics" => run_metrics_cli,
        "leave" => run_leave_cli,
        "set" => run_set_cli,
        "get" => run_get_cli,
        "delete" => run_delete_cli,
        "commit" => run_commit_cli,
        "rollback" => run_rollback_cli,
        "merge" => run_merge_cli,
        "search" => run_search_cli,
        "schema" => run_schema_cli,
        "schedule" => run_schedule_cli,
        "gateway" => run_gateway_cli,
        _ => panic!("Subcommand {} is unknown", subcommand),
    };

    if let Err(ref e) = run_cli(options) {
        let stderr = &mut std::io::stderr();
        let errmsg = "Error writing to stderr";
        writeln!(stderr, "{}", e).expect(errmsg);
        std::process::exit(1);
    }
}
