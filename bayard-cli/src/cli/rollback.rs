use clap::ArgMatches;

use bayard_client::index::client::IndexClient;

pub fn run_rollback_cli(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.rollback()
}
