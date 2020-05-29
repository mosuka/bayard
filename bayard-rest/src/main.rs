#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg};

use bayard_common::log::set_logger;
use bayard_rest::rest::server::{run, run_tls};

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
            Arg::with_name("INDEX_ADDRESS")
                .help("Index service address.")
                .short("i")
                .long("index-address")
                .value_name("ADDRESS")
                .default_value("0.0.0.0:5000")
                .takes_value(true),
        )
        // .arg(
        //     Arg::with_name("CORS_ORIGIN")
        //         .help("Add an origin that are allowed to make requests.")
        //         .short("o")
        //         .long("cors-origin")
        //         .value_name("ORIGIN")
        //         .takes_value(true),
        // )
        // .arg(
        //     Arg::with_name("CORS_METHODS")
        //         .help("Set a list of methods which the allowed origins are allowed to access for requests.")
        //         .short("m")
        //         .long("cors-method")
        //         .value_name("METHODS")
        //         .takes_value(true)
        //         .multiple(true)
        //         .use_delimiter(true)
        //         .require_delimiter(true)
        //         .value_delimiter(","),
        // )
        // .arg(
        //     Arg::with_name("CORS_HEADERS")
        //         .help("Set a list of header field names which can be used when this resource is accessed by allowed origins.")
        //         .short("l")
        //         .long("cors-headers")
        //         .value_name("HEADERS")
        //         .takes_value(true)
        //         .multiple(true)
        //         .use_delimiter(true)
        //         .require_delimiter(true)
        //         .value_delimiter(","),
        // )
        .arg(
            Arg::with_name("CERT_FILE")
                .help("Path to the TLS certificate file.")
                .short("c")
                .long("cert-file")
                .value_name("PATH")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("KEY_FILE")
                .help("Path to the TLS key file.")
                .short("k")
                .long("key-file")
                .value_name("PATH")
                .takes_value(true),
        );

    let matches = app.get_matches();

    let host = matches.value_of("HOST").unwrap();
    let port = matches.value_of("PORT").unwrap().parse::<u16>().unwrap();
    let index_address = matches.value_of("INDEX_ADDRESS").unwrap();
    // let mut cors_origin = "".to_string();
    // if let Some(_cors_origin) = matches.value_of("CORS_ORIGIN") {
    //     cors_origin = _cors_origin.to_string();
    // }
    // let mut cors_methods = Vec::new();
    // if let Some(_cors_methods) = matches.values_of("CORS_METHODS") {
    //     _cors_methods
    //         .map(|s| cors_methods.push(s.to_string()))
    //         .count();
    // }
    // let mut cors_headers = Vec::new();
    // if let Some(_cors_headers) = matches.values_of("CORS_HEADERS") {
    //     _cors_headers
    //         .map(|s| cors_headers.push(s.to_string()))
    //         .count();
    // }
    let mut cert_file = "";
    if let Some(_cert_file) = matches.value_of("CERT_FILE") {
        cert_file = _cert_file;
    }
    let mut key_file = "";
    if let Some(_key_file) = matches.value_of("KEY_FILE") {
        key_file = _key_file;
    }

    let rest_addr = format!("{}:{}", host, port);
    let rest_address = rest_addr.as_ref();

    if cert_file != "" && key_file != "" {
        if let Err(_e) = run_tls(rest_address, index_address, cert_file, key_file) {
            std::process::exit(1);
        }
    } else {
        if let Err(_e) = run(rest_address, index_address) {
            std::process::exit(1);
        }
    }

    Ok(())
}
