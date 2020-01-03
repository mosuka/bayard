extern crate bayard;
#[macro_use]
extern crate clap;
extern crate num_cpus;

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
use bayard::cmd::put::run_put_cli;
use bayard::cmd::rollback::run_rollback_cli;
use bayard::cmd::schedule::run_schedule_cli;
use bayard::cmd::schema::run_schema_cli;
use bayard::cmd::search::run_search_cli;
use bayard::cmd::serve::run_serve_cli;

fn main() {
    let cpus = num_cpus::get().to_owned();
    let threads;
    if cpus > 1 {
        threads = format!("{}", cpus - 1);
    } else {
        threads = format!("{}", cpus);
    }

    let app = App::new(crate_name!())
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .about("The `bayard` CLI manages server, cluster and index.")
        .help_message("Prints help information.")
        .version_message("Prints version information.")
        .version_short("v")
        .subcommand(
            SubCommand::with_name("serve")
                .name("serve")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("The `bayard serve` CLI starts the server.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("ID")
                        .help("Server ID. Must specify a numeric ID that is unique within the cluster. If not specified, use the default ID.")
                        .short("i")
                        .long("id")
                        .value_name("ID")
                        .default_value("1")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("HOST")
                        .help("Host address. Must specify the host name or IP address. If not specified, use the default address.")
                        .short("H")
                        .long("host")
                        .value_name("HOST")
                        .default_value("0.0.0.0")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("PORT")
                        .help("Port number. This port is used for communication via gRPC. If not specified, use the default port.")
                        .short("P")
                        .long("port")
                        .value_name("PORT")
                        .default_value("5000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("PEERS")
                        .help("Server ID and addresses in an existing cluster separated by \",\". If specified, the server will join the cluster.")
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
                        .help("Data directory. Stores index, snapshots, and raft logs. If not specified, use the default directory.")
                        .short("d")
                        .long("data-directory")
                        .value_name("DATA_DIRECTORY")
                        .default_value("./data")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("SCHEMA_FILE")
                        .help("Schema file. Must specify An existing file name. If not specified, use the default schema file.")
                        .short("s")
                        .long("schema-file")
                        .value_name("SCHEMA_FILE")
                        .default_value("./etc/schema.json")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("INDEXER_THREADS")
                        .help("Number of indexer threads. If not specified, number of CPU cores - 1 will be used.")
                        .short("t")
                        .long("indexer-threads")
                        .value_name("INDEXER_THREADS")
                        .default_value(&threads)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("INDEXER_MEMORY_SIZE")
                        .help("Total memory size (in bytes) used by the indexer. It will be split for the different thread. If not specified, use the default.")
                        .short("m")
                        .long("indexer-memory-size")
                        .value_name("INDEXER_MEMORY_SIZE")
                        .default_value("1000000000")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("probe")
                .name("probe")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("The `bayard probe` CLI probes the server.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVER")
                        .help("Server address in an existing cluster.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("127.0.0.1:5000")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("peers")
                .name("peers")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("The `bayard peers` CLI shows the peer addresses of the cluster that the specified server is joining.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVER")
                        .help("Server address in an existing cluster.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("127.0.0.1:5000")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("metrics")
                .name("metrics")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("The `bayard metrics` CLI shows the server metrics of the specified server. The metrics are output in Prometheus exposition format.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVER")
                        .help("Server address in an existing cluster.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("127.0.0.1:5000")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("leave")
                .name("leave")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("The `bayard leave` CLI removes the server with the specified ID from the cluster that the specified server is joining.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
                        .help("Node ID to be removed from the cluster that specified server is joining.")
                        .short("i")
                        .long("id")
                        .value_name("ID")
                        .default_value("1")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("get")
                .name("get")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("The `bayard get` CLI gets a document with the specified ID.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
                        .help("A unique value that identifies the document in the index.")
                        .short("i")
                        .long("id")
                        .value_name("ID")
                        .required(true)
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("put")
                .name("put")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("The `bayard put` CLI puts a document with the specified ID and field. If specify an existing ID, it will be overwritten with the new document.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
                    Arg::with_name("BULK")
                        .help("A flag indicating whether or not to put documents in bulk.")
                        .short("b")
                        .long("bulk"),
                )
                .arg(
                    Arg::with_name("ID")
                        .help("A unique value that identifies the document in the index. If specified, the existing document ID in the document is overwritten.")
                        .short("i")
                        .long("id")
                        .value_name("ID")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("FILE")
                        .help("File path that document(s) expressed in JSON or JSONL format.")
                        .short("f")
                        .long("file")
                        .value_name("FILE")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("delete")
                .name("delete")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("The `bayard delete` CLI deletes a document with the specified ID.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
                    Arg::with_name("BULK")
                        .help("A flag indicating whether or not to delete documents in bulk.")
                        .short("b")
                        .long("bulk"),
                )
                .arg(
                    Arg::with_name("ID")
                        .help("A unique value that identifies the document in the index.")
                        .short("i")
                        .long("id")
                        .value_name("ID")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("FILE")
                        .help("File path that delete document(s) expressed in JSONL format.")
                        .short("f")
                        .long("file")
                        .value_name("FILE")
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("commit")
                .name("commit")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("The `bayard commit` CLI commits updates made to the index.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
                .about("The `bayard rollback` CLI rolls back any updates made to the index to the last committed state.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
                .about("The `bayard merge` CLI merges fragmented segments in the index.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
                .about("The `bayard search` CLI searches documents from the index.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
                    Arg::with_name("FROM")
                        .help("Start position of fetching results. If not specified, use default value.")
                        .short("f")
                        .long("from")
                        .value_name("FROM")
                        .default_value("0")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("LIMIT")
                        .help("Limitation of amount that document to be returned. If not specified, use default value.")
                        .short("l")
                        .long("limit")
                        .value_name("LIMIT")
                        .default_value("10")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("EXCLUDE_COUNT")
                        .help("A flag indicating whether or not to exclude hit count in the search results.")
                        .short("c")
                        .long("exclude-count"),
                )
                .arg(
                    Arg::with_name("EXCLUDE_DOCS")
                        .help("A flag indicating whether or not to exclude hit documents in the search results")
                        .short("d")
                        .long("exclude-docs"),
                )
                .arg(
                    Arg::with_name("FACET_FIELD")
                        .help("Hierarchical facet field name.")
                        .short("F")
                        .long("facet-field")
                        .value_name("FACET_FIELD")
                        .default_value("")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("FACET_PREFIX")
                        .help("Hierarchical facet field value prefix.")
                        .short("V")
                        .long("facet-prefix")
                        .value_name("FACET_PREFIX")
                        .multiple(true)
                        .number_of_values(1)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("QUERY")
                        .help("Query string to search the index.")
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
                .about("The `bayard schema` CLI shows the index schema that the server applied.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
                .about("The `bayard schedule` CLI starts the job scheduler.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
                        .help("Schedule for automatic commit in a cron-like format. If not specified, use default schedule.")
                        .short("c")
                        .long("commit")
                        .value_name("COMMIT_SCHEDULE")
                        .default_value("0/10 * * * * * *")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("MERGE_SCHEDULE")
                        .help("Schedule for automatic merge in a cron-like format. If not specified, use default schedule.")
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
                .about("The `bayard gateway` CLI starts a gateway for access the server over HTTP.")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("HOST")
                        .help("Host address. Must specify the host name or IP address. If not specified, use the default address.")
                        .short("H")
                        .long("host")
                        .value_name("HOST")
                        .default_value("0.0.0.0")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("PORT")
                        .help("Port number. This port is used for communication via HTTP. If not specified, use the default port.")
                        .short("P")
                        .long("port")
                        .value_name("PORT")
                        .default_value("8000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("SERVERS")
                        .help("Server addresses in an existing cluster separated by \",\". If not specified, use default servers.")
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
        "put" => run_put_cli,
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
