use clap::ArgMatches;

use bayard_client::index::client::IndexClient;

use crate::log::set_logger;

pub fn run_commit_cli(matches: &ArgMatches) -> Result<(), std::io::Error> {
    set_logger();

    let server = matches.value_of("SERVER").unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.commit()
}
