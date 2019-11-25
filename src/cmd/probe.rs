use clap::ArgMatches;

use crate::client::client::{create_client, Clerk};
use crate::util::log::set_logger;

pub fn run_probe_cli(matches: &ArgMatches) -> Result<(), String> {
    set_logger();

    let servers: Vec<_> = matches
        .values_of("SERVERS")
        .unwrap()
        .map(|addr| create_client(addr))
        .collect();

    let client_id = rand::random();

    let mut client = Clerk::new(&servers, client_id);
    let value = client.probe();
    print!("{}", value);

    Ok(())
}
