use std::io;
use std::sync::Mutex;

use actix_server::Server;
use actix_web::{middleware, web, App, HttpServer};

use bayard_client::index::client::IndexClient;

use crate::rest::handler::{
    bulk_delete, bulk_set, commit, delete, get, merge, rollback, schema, search, set, status,
};

pub struct AppData {
    pub index_client: Mutex<IndexClient>,
}

pub type AppState = web::Data<AppData>;

pub struct RestServer {
    server: Server,
}

impl RestServer {
    pub fn new(address: &str, index_server_address: &str, worker_num: usize) -> RestServer {
        let index_client = IndexClient::new(index_server_address);
        let app_data = web::Data::new(AppData {
            index_client: Mutex::new(index_client),
        });

        let server = HttpServer::new(move || {
            App::new()
                .app_data(app_data.clone())
                .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
                .wrap(middleware::Compress::default())
                .wrap(middleware::Logger::default())
                .service(get)
                .service(set)
                .service(delete)
                .service(bulk_set)
                .service(bulk_delete)
                .service(commit)
                .service(rollback)
                .service(merge)
                .service(schema)
                .service(search)
                .service(status)
        })
        .bind(address)
        .unwrap()
        .workers(worker_num)
        .run();

        RestServer { server }
    }

    pub async fn shutdown(&mut self) -> io::Result<()> {
        Ok(self.server.stop(true).await)
    }
}
