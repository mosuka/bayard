use clap::ArgMatches;

use crate::client::client::{create_client, Clerk};
use crate::util::log::set_log_level;

pub fn run_set_cli(matches: &ArgMatches) -> Result<(), String> {
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

    let key = matches.value_of("KEY").unwrap();
    let value = matches.value_of("VALUE").unwrap();

    let client_id = rand::random();

    let mut client = Clerk::new(&servers, client_id);
    client.put(key, value);

    Ok(())
}
