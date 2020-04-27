use clap::ArgMatches;
use iron::{Chain, Iron};
use logger::Logger;
use persistent::Write;
use router::Router;

use bayard_client::index::client::IndexClient;

use crate::handler::bulk_delete::bulk_delete;
use crate::handler::bulk_set::bulk_set;
use crate::handler::commit::commit;
use crate::handler::delete::delete;
use crate::handler::get::get;
use crate::handler::merge::merge;
use crate::handler::metrics::metrics;
use crate::handler::rollback::rollback;
use crate::handler::schema::schema;
use crate::handler::search::search;
use crate::handler::set::set;
use crate::handler::status::status;
use crate::handler::Client;
use crate::log::set_http_logger;

pub fn run_start_cli(matches: &ArgMatches) -> Result<(), std::io::Error> {
    set_http_logger();

    let host = matches.value_of("HOST").unwrap();
    let port = matches.value_of("PORT").unwrap().parse::<u16>().unwrap();
    let server = matches.value_of("SERVER").unwrap();

    let (logger_before, logger_after) = Logger::new(None);

    let index_client = IndexClient::new(server);

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
    router.get("/v1/metrics", metrics, "metrics");

    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link(Write::<Client>::both(index_client));
    chain.link_after(logger_after);

    Iron::new(chain).http(format!("{}:{}", host, port)).unwrap();

    Ok(())
}
