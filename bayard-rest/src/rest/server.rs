use std::io::{Error, ErrorKind};

use iron::{Chain, Iron, Listening};
use log::*;
use logger::Logger;
use persistent::Write;
use router::Router;

use bayard_client::index::client::IndexClient;

use crate::rest::Client;
use crate::rest::handler::{bulk_delete, bulk_set, commit, delete, get, merge, rollback, schema, search, set, status};

pub struct RestServer {
    address: String,
    listening: Listening,
}

impl RestServer {
    pub fn new(address: &str, server: &str) -> RestServer {
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

        let listening = Iron::new(chain).http(address).unwrap();
        info!("start rest service on {}", address);

        RestServer {
            address: address.to_string(),
            listening,
        }
    }

    pub fn shutdown(&mut self) -> Result<(), std::io::Error> {
        match self.listening.close() {
            Ok(_) => {
                info!("stop rest service on {}", self.address);
                Ok(())
            }
            Err(e) => {
                error!(
                    "failed stop rest service on {}: error={:?}",
                    self.address, e
                );
                Err(Error::new(ErrorKind::Other, e))
            }
        }
    }
}
