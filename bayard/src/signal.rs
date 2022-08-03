use signal_hook::{
    consts::signal::{SIGINT, SIGQUIT, SIGTERM},
    iterator::{exfiltrator::WithOrigin, SignalsInfo},
};
use tokio::sync::watch::Sender;
use tracing::{debug, error, info};

pub fn handle_signals(tx_signal: Sender<()>) {
    info!("Starting signal handler.");
    let mut signals = SignalsInfo::<WithOrigin>::new(&[SIGINT, SIGTERM, SIGQUIT])
        .expect("Failed to create signals info");
    for signal_info in &mut signals {
        debug!(?signal_info, "Received signal.");
        match signal_info.signal {
            SIGINT | SIGTERM | SIGQUIT => {
                if let Err(error) = tx_signal.send(()) {
                    error!(?error, "Failed to send stop signal.");
                }
                break;
            }
            _ => {}
        }
    }
}
