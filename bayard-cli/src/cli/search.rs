use clap::ArgMatches;

use bayard_client::index::client::IndexClient;

pub fn run_search_cli(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let from = matches.value_of("FROM").unwrap().parse::<u64>().unwrap();
    let limit = matches.value_of("LIMIT").unwrap().parse::<u64>().unwrap();
    let exclude_count = matches.is_present("EXCLUDE_COUNT");
    let exclude_docs = matches.is_present("EXCLUDE_DOCS");
    let facet_field = matches.value_of("FACET_FIELD").unwrap();
    let mut facet_prefixes: Vec<String> = Vec::new();
    if let Some(_facet_prefixes) = matches.values_of("FACET_PREFIX") {
        _facet_prefixes
            .map(|s| facet_prefixes.push(s.to_string()))
            .count();
    }
    let query = matches.value_of("QUERY").unwrap();

    let mut index_client = IndexClient::new(server);

    match index_client.search(
        query,
        from,
        limit,
        exclude_count,
        exclude_docs,
        facet_field,
        facet_prefixes,
    ) {
        Ok(v) => {
            println!("{}", v);
            Ok(())
        }
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}
