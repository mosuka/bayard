use std::io::{Error, ErrorKind};

use iron::{Chain, Iron, Listening};
use logger::Logger;
use router::Router;

use crate::metric::handler::metrics;

pub struct MetricsServer {
    listening: Listening,
}

impl MetricsServer {
    pub fn new(address: &str) -> MetricsServer {
        let (logger_before, logger_after) = Logger::new(None);
        let mut router = Router::new();
        router.get("/metrics", metrics, "metrics");

        let mut chain = Chain::new(router);
        chain.link_before(logger_before);
        chain.link_after(logger_after);

        let listening = Iron::new(chain).http(address).unwrap();

        MetricsServer { listening }
    }

    pub fn shutdown(&mut self) -> Result<(), std::io::Error> {
        match self.listening.close() {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
        }
    }
}
