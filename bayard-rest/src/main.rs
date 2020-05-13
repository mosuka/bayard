#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg};
use crossbeam_channel::select;
use log::*;

use bayard_common::log::set_logger;
use bayard_common::signal::sigterm_channel;
use bayard_rest::rest::server::RestServer;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    set_logger();

    let threads = format!("{}", num_cpus::get().to_owned());

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
                .help("Hostname or IP address.")
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
            Arg::with_name("INDEX_SERVER")
                .help("Index service address.")
                .short("i")
                .long("index-server")
                .value_name("ADDRESS")
                .default_value("0.0.0.0:5000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("HTTP_WORKER_THREADS")
                .help("Number of HTTP worker threads. By default http server uses number of available logical cpu as threads count.")
                .short("w")
                .long("worker-threads")
                .value_name("THREADS")
                .default_value(&threads)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("CORS_ORIGIN")
                .help("Add an origin that are allowed to make requests.")
                .short("o")
                .long("cors-origin")
                .value_name("ORIGIN")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("CORS_METHODS")
                .help("Set a list of methods which the allowed origins are allowed to access for requests.")
                .short("m")
                .long("cors-method")
                .value_name("METHODS")
                .takes_value(true)
                .multiple(true)
                .use_delimiter(true)
                .require_delimiter(true)
                .value_delimiter(","),
        )
        .arg(
            Arg::with_name("CORS_HEADERS")
                .help("Set a list of header field names which can be used when this resource is accessed by allowed origins.")
                .short("l")
                .long("cors-headers")
                .value_name("HEADERS")
                .takes_value(true)
                .multiple(true)
                .use_delimiter(true)
                .require_delimiter(true)
                .value_delimiter(","),
        );

    let matches = app.get_matches();

    let host = matches.value_of("HOST").unwrap();
    let port = matches.value_of("PORT").unwrap().parse::<u16>().unwrap();
    let server = matches.value_of("SERVER").unwrap();
    let http_worker_threads = matches
        .value_of("HTTP_WORKER_THREADS")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let mut cors_origin = "".to_string();
    if let Some(_cors_origin) = matches.value_of("CORS_ORIGIN") {
        cors_origin = _cors_origin.to_string();
    }
    let mut cors_methods = Vec::new();
    if let Some(_cors_methods) = matches.values_of("CORS_METHODS") {
        _cors_methods
            .map(|s| cors_methods.push(s.to_string()))
            .count();
    }
    let mut cors_headers = Vec::new();
    if let Some(_cors_headers) = matches.values_of("CORS_HEADERS") {
        _cors_headers
            .map(|s| cors_headers.push(s.to_string()))
            .count();
    }

    let rest_address = format!("{}:{}", host, port);

    let mut rest_server;
    if cors_origin == "" {
        rest_server = RestServer::new(rest_address.as_str(), server, http_worker_threads);
    } else {
        info!("enable CORS: origin={:?}, methods={:?}, headers={:?}", cors_origin, cors_methods, cors_headers);
        rest_server = RestServer::new_cors(rest_address.as_str(), server, http_worker_threads, cors_origin, cors_methods, cors_headers);
    }
    info!("start rest service on {}", rest_address.as_str());

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

    match rest_server.shutdown().await {
        Ok(_) => {
            info!("stop rest service on {}:{}", host, port);
            Ok(())
        }
        Err(e) => {
            error!(
                "failed to stop rest service on {}:{}: error={}",
                host, port, e
            );
            Err(e)
        }
    }
}
