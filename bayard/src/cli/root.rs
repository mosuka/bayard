use clap::{App, AppSettings, Arg, SubCommand};
use num_cpus;

use crate::cli::bulk_delete::run_bulk_delete_cli;
use crate::cli::bulk_set::run_bulk_set_cli;
use crate::cli::commit::run_commit_cli;
use crate::cli::delete::run_delete_cli;
use crate::cli::get::run_get_cli;
use crate::cli::leave::run_leave_cli;
use crate::cli::merge::run_merge_cli;
use crate::cli::metrics::run_metrics_cli;
use crate::cli::rollback::run_rollback_cli;
use crate::cli::schema::run_schema_cli;
use crate::cli::search::run_search_cli;
use crate::cli::set::run_set_cli;
use crate::cli::start::run_start_cli;
use crate::cli::status::run_status_cli;

pub fn run_root_cli() -> Result<(), std::io::Error> {
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
        .about("Manage Bayard index server")
        .help_message("Prints help information.")
        .version_message("Prints version information.")
        .version_short("v")
        .subcommand(
            SubCommand::with_name("start")
                .name("start")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Start index server")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("ID")
                        .help("Node ID.")
                        .value_name("ID")
                        .takes_value(true),
                )
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
                    Arg::with_name("RAFT_PORT")
                        .help("Raft service port number.")
                        .short("r")
                        .long("raft-port")
                        .value_name("RAFT_PORT")
                        .default_value("7000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("INDEX_PORT")
                        .help("Index service port number")
                        .short("i")
                        .long("index-port")
                        .value_name("INDEX_PORT")
                        .default_value("5000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("PEER_RAFT_ADDRESS")
                        .help("Raft address of a peer node running in an existing cluster.")
                        .short("p")
                        .long("peer-raft-address")
                        .value_name("IP:PORT")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("DATA_DIRECTORY")
                        .help("Data directory. Stores index, snapshots, and raft logs.")
                        .short("d")
                        .long("data-directory")
                        .value_name("DATA_DIRECTORY")
                        .default_value("./data")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("SCHEMA_FILE")
                        .help("Schema file. Must specify An existing file name.")
                        .short("s")
                        .long("schema-file")
                        .value_name("SCHEMA_FILE")
                        .default_value("./etc/schema.json")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("TOKENIZER_FILE")
                        .help("Tokenizer file. Must specify An existing file name.")
                        .short("T")
                        .long("tokenizer-file")
                        .value_name("TOKENIZER_FILE")
                        .default_value("./etc/tokenizer.json")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("INDEXER_THREADS")
                        .help("Number of indexer threads.")
                        .short("t")
                        .long("indexer-threads")
                        .value_name("INDEXER_THREADS")
                        .default_value(&threads)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("INDEXER_MEMORY_SIZE")
                        .help("Total memory size (in bytes) used by the indexer.")
                        .short("m")
                        .long("indexer-memory-size")
                        .value_name("INDEXER_MEMORY_SIZE")
                        .default_value("1000000000")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("leave")
                .name("leave")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Delete node from the cluster")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVER")
                        .help("Raft service address.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("0.0.0.0:7000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ID")
                        .help("Node ID to be removed from the cluster.")
                        .value_name("ID")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .name("get")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Get document from index server")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVER")
                        .help("Index service address.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("0.0.0.0:5000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ID")
                        .help("A unique ID that identifies the document in the index server.")
                        .value_name("ID")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("set")
                .name("set")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Set document to index server")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVER")
                        .help("Index service address.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("0.0.0.0:5000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ID")
                        .help("A unique ID that identifies the document in the index server.")
                        .value_name("ID")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("FIELDS")
                        .help("Fields of document to be indexed.")
                        .value_name("FIELDS")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .name("delete")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Delete document from index server")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVER")
                        .help("Index service address.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("0.0.0.0:5000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("ID")
                        .help("A unique ID that identifies the document in the index server.")
                        .value_name("ID")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("bulk-set")
                .name("bulk-set")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Set documents to index server in bulk")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVER")
                        .help("Index service address.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("0.0.0.0:5000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("DOCS")
                        .help("Document containing the unique ID to be indexed.")
                        .value_name("DOCS")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("bulk-delete")
                .name("bulk-delete")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Delete documents from index server in bulk")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVER")
                        .help("Index service address.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("0.0.0.0:5000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("DOCS")
                        .help("Document containing the unique ID to be indexed.")
                        .value_name("DOCS")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("commit")
                .name("commit")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Commit index")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
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
        .subcommand(
            SubCommand::with_name("rollback")
                .name("rollback")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Rollback index")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
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
        .subcommand(
            SubCommand::with_name("merge")
                .name("merge")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Merge index")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
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
        .subcommand(
            SubCommand::with_name("schema")
                .name("schema")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Shows index schema that applied")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
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
        .subcommand(
            SubCommand::with_name("search")
                .name("search")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Search documents from index server")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
                .arg(
                    Arg::with_name("SERVER")
                        .help("Index service address.")
                        .short("s")
                        .long("server")
                        .value_name("IP:PORT")
                        .default_value("0.0.0.0:5000")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("FROM")
                        .help("Start position of fetching results.")
                        .short("f")
                        .long("from")
                        .value_name("FROM")
                        .default_value("0")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("LIMIT")
                        .help("Limitation of amount that document to be returned.")
                        .short("l")
                        .long("limit")
                        .value_name("LIMIT")
                        .default_value("10")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("EXCLUDE_COUNT")
                        .help("A flag for exclude hit count in the search result.")
                        .short("c")
                        .long("exclude-count"),
                )
                .arg(
                    Arg::with_name("EXCLUDE_DOCS")
                        .help("A flag for exclude hit documents in the search result.")
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
                        .help("Query string for search the index.")
                        .value_name("QUERY")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("status")
                .name("status")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Shows system status")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
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
        .subcommand(
            SubCommand::with_name("metrics")
                .name("metrics")
                .setting(AppSettings::DeriveDisplayOrder)
                .version(crate_version!())
                .author(crate_authors!())
                .about("Shows system metrics")
                .help_message("Prints help information.")
                .version_message("Prints version information.")
                .version_short("v")
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
        "leave" => run_leave_cli,
        "search" => run_search_cli,
        "get" => run_get_cli,
        "set" => run_set_cli,
        "delete" => run_delete_cli,
        "bulk-set" => run_bulk_set_cli,
        "bulk-delete" => run_bulk_delete_cli,
        "commit" => run_commit_cli,
        "rollback" => run_rollback_cli,
        "merge" => run_merge_cli,
        "schema" => run_schema_cli,
        "metrics" => run_metrics_cli,
        "status" => run_status_cli,
        _ => panic!("Subcommand {} is unknown", subcommand),
    };

    run_cli(options)
}
