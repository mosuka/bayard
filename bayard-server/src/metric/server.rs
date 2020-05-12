use std::io;

use actix_server::Server;
use actix_web::{middleware, App, HttpServer};

use crate::metric::handler::metrics;

pub struct MetricsServer {
    server: Server,
}

impl MetricsServer {
    pub fn new(address: &str, worker_num: usize) -> MetricsServer {
        let server = HttpServer::new(move || {
            App::new()
                .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
                .wrap(middleware::Compress::default())
                .wrap(middleware::Logger::default())
                .service(metrics)
        })
        .bind(address)
        .unwrap()
        .workers(worker_num)
        .run();

        MetricsServer { server }
    }

    pub async fn shutdown(&mut self) -> io::Result<()> {
        Ok(self.server.stop(true).await)
    }
}
