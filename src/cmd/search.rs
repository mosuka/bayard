use clap::ArgMatches;

use crate::client::client::{create_client, Clerk};
use crate::util::log::set_logger;

pub fn run_search_cli(matches: &ArgMatches) -> Result<(), String> {
    set_logger();

    let servers: Vec<_> = matches
        .values_of("SERVERS")
        .unwrap()
        .map(|addr| create_client(addr))
        .collect();
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
    //    let mut facets: Vec<String> = Vec::new();
    //    if let Some(_facets) = matches.values_of("FACET") {
    //        _facets.map(|s| {
    //            facets.push(s.to_string())
    //        }
    //        ).count();
    //    }

    let query = matches.value_of("QUERY").unwrap();

    let client_id = rand::random();

    let mut client = Clerk::new(&servers, client_id);
    let value = client.search(
        query,
        from,
        limit,
        exclude_count,
        exclude_docs,
        facet_field,
        facet_prefixes,
    );
    //    let value = client.search(query, from, limit, exclude_count, exclude_docs, facets);
    print!("{}", value);

    Ok(())
}
