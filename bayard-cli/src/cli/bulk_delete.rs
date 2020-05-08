use clap::ArgMatches;

use bayard_client::index::client::IndexClient;

pub fn run_bulk_delete_cli(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let docs = matches.value_of("DOCS").unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.bulk_delete(docs.to_string())
}
