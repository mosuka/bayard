use clap::ArgMatches;

use bayard_client::index::client::IndexClient;

pub fn run_get_cli(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let id = matches.value_of("ID").unwrap();

    let mut index_client = IndexClient::new(server);

    match index_client.get(id.to_string()) {
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
