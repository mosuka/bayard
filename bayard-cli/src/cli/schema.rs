use clap::ArgMatches;

use bayard_client::index::client::IndexClient;

pub fn run_schema_cli(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();

    let mut index_client = IndexClient::new(server);

    match index_client.schema() {
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