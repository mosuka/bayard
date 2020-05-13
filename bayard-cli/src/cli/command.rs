use clap::{App, AppSettings, Arg, SubCommand};

use crate::cli::subcommand::{
    bulk_delete, bulk_set, commit, delete, get, leave, merge, rollback, schema, search, set, status,
};

pub fn root() -> Result<(), std::io::Error> {
    let app = App::new(crate_name!())
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Bayard command-line interface")
        .help_message("Prints help information.")
        .version_message("Prints version information.")
        .version_short("v")
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
        .get_matches();

    let (subcommand, some_options) = app.subcommand();
    let options = some_options.unwrap();
    let run_cli = match subcommand {
        "leave" => leave,
        "search" => search,
        "get" => get,
        "set" => set,
        "delete" => delete,
        "bulk-set" => bulk_set,
        "bulk-delete" => bulk_delete,
        "commit" => commit,
        "rollback" => rollback,
        "merge" => merge,
        "schema" => schema,
        "status" => status,
        _ => panic!("Subcommand {} is unknown", subcommand),
    };

    run_cli(options)
}
