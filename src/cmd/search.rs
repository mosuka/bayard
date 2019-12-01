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
    let query = matches.value_of("QUERY").unwrap();

    let client_id = rand::random();

    let mut client = Clerk::new(&servers, client_id);
    let value = client.search(query, from, limit);
    print!("{}", value);

    Ok(())
}
