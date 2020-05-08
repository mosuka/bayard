#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg};
use crossbeam_channel::select;
use iron::{Chain, Iron};
use log::*;
use logger::Logger;
use persistent::Write;
use router::Router;

use bayard_client::index::client::IndexClient;
use bayard_common::log::set_logger;
use bayard_common::signal::sigterm_channel;
use bayard_rest::handler::bulk_delete::bulk_delete;
use bayard_rest::handler::bulk_set::bulk_set;
use bayard_rest::handler::commit::commit;
use bayard_rest::handler::delete::delete;
use bayard_rest::handler::get::get;
use bayard_rest::handler::merge::merge;
use bayard_rest::handler::rollback::rollback;
use bayard_rest::handler::schema::schema;
use bayard_rest::handler::search::search;
use bayard_rest::handler::set::set;
use bayard_rest::handler::status::status;
use bayard_rest::handler::Client;

fn main() -> Result<(), std::io::Error> {
    set_logger();

    let app = App::new(crate_name!())
        .setting(AppSettings::DeriveDisplayOrder)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Bayard REST server")
        .help_message("Prints help information.")
        .version_message("Prints version information.")
        .version_short("v")
        .arg(
            Arg::with_name("HOST")
                .help("Node address.")
                .short("H")
                .long("host")
                .value_name("HOST")
                .default_value("0.0.0.0")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("PORT")
                .help("HTTP service port number.")
                .short("p")
                .long("port")
                .value_name("PORT")
                .default_value("8000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("SERVER")
                .help("Index service address.")
                .short("s")
                .long("server")
                .value_name("IP:PORT")
                .default_value("0.0.0.0:5000")
                .takes_value(true),
        );

    let matches = app.get_matches();

    let host = matches.value_of("HOST").unwrap();
    let port = matches.value_of("PORT").unwrap().parse::<u16>().unwrap();
    let server = matches.value_of("SERVER").unwrap();

    let index_client = IndexClient::new(server);

    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/v1/documents/:id", get, "get");
    router.put("/v1/documents/:id", set, "set");
    router.delete("/v1/documents/:id", delete, "delete");
    router.put("/v1/documents", bulk_set, "bulk_set");
    router.delete("/v1/documents", bulk_delete, "bulk_delete");
    router.get("/v1/commit", commit, "commit");
    router.get("/v1/rollback", rollback, "rollback");
    router.get("/v1/merge", merge, "merge");
    router.get("/v1/schema", schema, "schema");
    router.post("/v1/search", search, "search");
    router.get("/v1/status", status, "status");

    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link(Write::<Client>::both(index_client));
    chain.link_after(logger_after);

    let mut rest_server = Iron::new(chain).http(format!("{}:{}", host, port)).unwrap();
    info!("start rest service on {}:{}", host, port);

    // Wait for signals for termination (SIGINT, SIGTERM).
    let sigterm_receiver = sigterm_channel().unwrap();
    loop {
        select! {
            recv(sigterm_receiver) -> _ => {
                info!("receive signal");
                break;
            }
        }
    }

    match rest_server.close() {
        Ok(_) => {
            info!("stop metrics service on {}:{}", host, port);
        }
        Err(e) => error!("{}", e),
    }

    info!("exit");

    Ok(())
}
