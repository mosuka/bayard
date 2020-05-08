use clap::ArgMatches;

use bayard_client::index::client::IndexClient;

pub fn run_delete_cli(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let id = matches.value_of("ID").unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.delete(id.to_string())
}
