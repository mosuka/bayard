use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use bayard::{
    client::Client,
    cluster::{membership::Membership, metadata::Metadata},
    index::{metastore::Metastore, INDICES_DIR},
    node::Node,
    server::{handle_grpc_server, handle_http_server},
    signal::handle_signals,
};
use clap::{AppSettings, Parser};
use tokio::{
    sync::{watch, RwLock},
    time::{sleep, Duration},
};

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None, setting = AppSettings::DeriveDisplayOrder)]
struct Args {
    /// The bind address that should be bound to for internal cluster communications.
    #[clap(
        short = 'b',
        long = "bind-address",
        default_value = "0.0.0.0:2000",
        value_name = "BIND_ADDRESS"
    )]
    bind_address: String,

    /// The advertise address is used to change the address that advertise to other nodes in the cluster. If it omitted, the bind address is used.
    #[clap(
        short = 'a',
        long = "advertise-address",
        value_name = "ADVERTISE_ADDRESS"
    )]
    advertise_address: Option<String>,

    /// The seed address is the address of another node in the cluster that is specified when joining the cluster.
    #[clap(short = 's', long = "seed-address", value_name = "SEED_ADDRESS")]
    seed_address: Option<String>,

    /// The gRPC address that should be bound to for internal cluster communications and client communications.
    #[clap(
        short = 'g',
        long = "grpc-address",
        default_value = "0.0.0.0:5000",
        value_name = "GRPC_ADDRESS"
    )]
    grpc_address: String,

    /// The HTTP address that should be bound to for client communications.
    #[clap(
        short = 'H',
        long = "http-address",
        default_value = "0.0.0.0:8000",
        value_name = "HTTP_ADDRESS"
    )]
    http_address: String,

    /// The data directory that should be store node list, indices and etc.
    #[clap(
        short = 'd',
        long = "data-directory",
        default_value = "/tmp/bayard",
        value_name = "DATA_DIRECTORY"
    )]
    data_directory: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    // Parse the command line arguments.
    let args = Args::parse();

    let bind_address = args.bind_address.parse::<SocketAddr>()?;

    let advertise_address = args
        .advertise_address
        .unwrap_or_else(|| args.bind_address.clone())
        .parse::<SocketAddr>()?;

    let seed_address = if let Some(a) = args.seed_address {
        Some(a.parse::<SocketAddr>()?)
    } else {
        None
    };

    let grpc_address = args.grpc_address.parse::<SocketAddr>()?;
    let http_address = args.http_address.parse::<SocketAddr>()?;

    // Create the signal handler.
    let (tx_signal, rx_signal) = watch::channel(());
    let signal_handler = tokio::spawn(async move {
        handle_signals(tx_signal);
    });

    // Create the node metadata.
    let metadata = Metadata {
        grpc_address: Some(grpc_address),
        http_address: Some(http_address),
    };

    // Create membership.
    let membership = Arc::new(
        Membership::new(
            bind_address,
            advertise_address,
            metadata,
            args.data_directory.clone(),
            seed_address,
        )
        .await?,
    );

    // Create index watcher.
    let metastore = Arc::new(Metastore::new(args.data_directory.join(INDICES_DIR)).await?);

    // Create the client.
    let client = Arc::new(Client::new(Arc::clone(&membership), Arc::clone(&metastore)).await);

    // Create the node.
    let node = Arc::new(RwLock::new(
        Node::new(membership, metastore, args.data_directory.join(INDICES_DIR)).await?,
    ));

    // Create the gRPC server handler.
    let rx_grpc_signal = rx_signal.clone();
    let grpc_server_handler = tokio::spawn(async move {
        handle_grpc_server(grpc_address, node, client, rx_grpc_signal).await;
    });

    sleep(Duration::from_secs(1)).await;

    // Create the HTTP server handler.
    let http_server_handler = tokio::spawn(async move {
        handle_http_server(http_address, grpc_address, rx_signal).await;
    });

    // Wait for handlers to finish.
    grpc_server_handler.await?;
    http_server_handler.await?;
    signal_handler.await?;

    Ok(())
}
