use std::fs::File;
use std::io;
use std::io::BufReader;
use std::iter::FromIterator;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_server::Server;
use actix_web::{middleware, web, App, HttpServer};
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{NoClientAuth, ServerConfig};

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

    pub fn new_cors(
        address: &str,
        index_server_address: &str,
        worker_num: usize,
        cors_origin: String,
        cors_methods: Vec<String>,
        cors_headers: Vec<String>,
    ) -> RestServer {
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
                .wrap(
                    Cors::new()
                        .allowed_origin(cors_origin.as_ref())
                        .allowed_methods(Vec::from_iter(cors_methods.iter().map(String::as_str)))
                        .allowed_headers(Vec::from_iter(cors_headers.iter().map(String::as_str)))
                        .finish(),
                )
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

    pub fn new_tls(
        address: &str,
        index_server_address: &str,
        worker_num: usize,
        cert_file: &str,
        key_file: &str,
    ) -> RestServer {
        let index_client = IndexClient::new(index_server_address);
        let app_data = web::Data::new(AppData {
            index_client: Mutex::new(index_client),
        });

        // load ssl keys
        let mut config = ServerConfig::new(NoClientAuth::new());
        let cert_reader = &mut BufReader::new(File::open(cert_file).unwrap());
        let key_reader = &mut BufReader::new(File::open(key_file).unwrap());
        let cert_chain = certs(cert_reader).unwrap();
        let mut keys = rsa_private_keys(key_reader).unwrap();
        config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

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
        .bind_rustls(address, config)
        .unwrap()
        .workers(worker_num)
        .run();

        RestServer { server }
    }

    pub fn new_cors_tls(
        address: &str,
        index_server_address: &str,
        worker_num: usize,
        cors_origin: String,
        cors_methods: Vec<String>,
        cors_headers: Vec<String>,
        cert_file: &str,
        key_file: &str,
    ) -> RestServer {
        let index_client = IndexClient::new(index_server_address);
        let app_data = web::Data::new(AppData {
            index_client: Mutex::new(index_client),
        });

        // load ssl keys
        let mut config = ServerConfig::new(NoClientAuth::new());
        let cert = &mut BufReader::new(File::open(cert_file).unwrap());
        let key = &mut BufReader::new(File::open(key_file).unwrap());
        let cert_chain = certs(cert).unwrap();
        let mut keys = rsa_private_keys(key).unwrap();
        config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

        let server = HttpServer::new(move || {
            App::new()
                .app_data(app_data.clone())
                .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
                .wrap(middleware::Compress::default())
                .wrap(middleware::Logger::default())
                .wrap(
                    Cors::new()
                        .allowed_origin(cors_origin.as_ref())
                        .allowed_methods(Vec::from_iter(cors_methods.iter().map(String::as_str)))
                        .allowed_headers(Vec::from_iter(cors_headers.iter().map(String::as_str)))
                        .finish(),
                )
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
        .bind_rustls(address, config)
        .unwrap()
        .workers(worker_num)
        .run();

        RestServer { server }
    }

    pub async fn shutdown(&mut self) -> io::Result<()> {
        Ok(self.server.stop(true).await)
    }
}
