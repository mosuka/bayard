use std::convert::TryFrom;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fs, io, sync};

use futures_util::{
    future::{ready, TryFutureExt},
    stream::{Stream, StreamExt, TryStreamExt},
};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use log::*;
use rustls::internal::pemfile;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

use bayard_client::index::client::IndexClient;
use bayard_common::signal::shutdown_signal;

use crate::rest::handler::{route, GenericError};

#[tokio::main]
pub async fn run(
    rest_address: &str,
    index_address: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let index_client = IndexClient::new(index_address);
    let address: SocketAddr = rest_address.parse().unwrap();

    let rest_server = Server::bind(&address).serve(make_service_fn(|_| {
        let index_client = index_client.clone();
        async move { Ok::<_, GenericError>(service_fn(move |req| route(req, index_client.clone()))) }
    }));

    let rest_server_graceful = rest_server.with_graceful_shutdown(shutdown_signal());
    info!("start rest service on {}", rest_address);

    match rest_server_graceful.await {
        Ok(_) => {
            info!("stop rest service on {}", rest_address);
            Ok(())
        }
        Err(e) => {
            error!(
                "failed to stop rest service on {}: error={}",
                rest_address, e
            );
            Err(Box::try_from(Error::new(ErrorKind::Other, e)).unwrap())
        }
    }
}

#[tokio::main]
pub async fn run_tls(
    rest_address: &str,
    index_address: &str,
    cert_file: &str,
    key_file: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let index_client = IndexClient::new(index_address);
    let address: SocketAddr = rest_address.parse().unwrap();

    let tls_cfg = {
        let certs = load_certs(cert_file)?;
        let key = load_private_key(key_file)?;
        let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        cfg.set_single_cert(certs, key)
            .map_err(|e| error(format!("{}", e)))?;
        cfg.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
        sync::Arc::new(cfg)
    };

    let mut tcp_listener = TcpListener::bind(&address).await.unwrap();
    let tls_acceptor = TlsAcceptor::from(tls_cfg);
    let incoming_tls_stream = tcp_listener
        .incoming()
        .map_err(|e| error(format!("Incoming failed: {:?}", e)))
        .and_then(move |s| {
            tls_acceptor.accept(s).map_err(|e| {
                error!("TLS Error: {:?}", e);
                error(format!("TLS Error: {:?}", e))
            })
        })
        .filter(|res| {
            // Ignore failed accepts
            ready(res.is_ok())
        })
        .boxed();

    let rest_server =
        Server::builder(HyperAcceptor {
            acceptor: incoming_tls_stream,
        })
            .serve(make_service_fn(|_| {
                let index_client = index_client.clone();
                async move {
                    Ok::<_, GenericError>(service_fn(move |req| route(req, index_client.clone())))
                }
            }));

    let rest_server_graceful = rest_server.with_graceful_shutdown(shutdown_signal());
    info!("start rest service on {}", rest_address);

    match rest_server_graceful.await {
        Ok(_) => {
            info!("stop rest service on {}", rest_address);
            Ok(())
        }
        Err(e) => {
            error!(
                "failed to stop rest service on {}: error={}",
                rest_address, e
            );
            Err(Box::try_from(Error::new(ErrorKind::Other, e)).unwrap())
        }
    }
}

// Load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<rustls::Certificate>> {
    // Open certificate file.
    let certfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    pemfile::certs(&mut reader).map_err(|_| error("failed to load certificate".into()))
}

// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = pemfile::rsa_private_keys(&mut reader)
        .map_err(|_| error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(error("expected a single private key".into()));
    }
    Ok(keys[0].clone())
}

fn error(err: String) -> Error {
    Error::new(ErrorKind::Other, err)
}

struct HyperAcceptor<'a> {
    acceptor: Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, io::Error>> + 'a>>,
}

impl hyper::server::accept::Accept for HyperAcceptor<'_> {
    type Conn = TlsStream<TcpStream>;
    type Error = io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        Pin::new(&mut self.acceptor).poll_next(cx)
    }
}
