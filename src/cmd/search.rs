use clap::ArgMatches;

use crate::client::client::{create_client, Clerk};
use crate::util::log::set_log_level;

pub fn run_search_cli(matches: &ArgMatches) -> Result<(), String> {
    set_log_level();
    env_logger::init();

    let mut servers = vec![];
    if let Some(addr_vec) = matches.values_of("SERVERS") {
        addr_vec
            .map(|addr| {
                servers.push(create_client(addr));
            })
            .count();
    }

    let query = matches.value_of("QUERY").unwrap();

    let client_id = rand::random();

    let mut client = Clerk::new(&servers, client_id);
    let value = client.search(query);
    print!("{}", value);

    Ok(())
}
